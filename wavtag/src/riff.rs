use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Write};

use crate::utils;

pub struct RiffChunk {
    pub header: ChunkType,
    pub data: Vec<u8>,
}

impl RiffChunk {
    pub fn len(&self) -> usize {
        self.data.len() // todo: investigate if this is actually valid given we have padded bytes.
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChunkType {
    Format,
    Data,
    Fact,
    Cue,
    Playlist,
    List,
    Label,
    Note,
    Acid,
    Instrument,
    Sampler,
    Unknown(String),
}

impl ChunkType {
    pub fn to_tag(self) -> [u8; 4] {
        match self {
            ChunkType::Format => *b"fmt ",
            ChunkType::Data => *b"data",
            ChunkType::Fact => *b"fact",
            ChunkType::Cue => *b"cue ",
            ChunkType::Playlist => *b"plst",
            ChunkType::List => *b"list",
            ChunkType::Label => *b"labl",
            ChunkType::Note => *b"note",
            ChunkType::Sampler => *b"smpl",
            ChunkType::Instrument => *b"inst",
            ChunkType::Acid => *b"acid",
            ChunkType::Unknown(tag) => {
                let tag = tag.as_bytes();
                [tag[0], tag[1], tag[2], tag[3]]
            }
        }
    }
}

fn header_to_rifftype(tag: [u8; 4]) -> ChunkType {
    match &tag {
        b"fmt " | b"FMT " => ChunkType::Format,
        b"data" | b"DATA" => ChunkType::Data,
        b"fact" | b"FACT" => ChunkType::Fact,
        b"cue " | b"CUE " => ChunkType::Cue,
        b"plst" | b"PLST" => ChunkType::Playlist,
        b"list" | b"LIST" => ChunkType::List,
        b"labl" | b"LABL" => ChunkType::Label,
        b"note" | b"NOTE" => ChunkType::Note,
        b"smpl" | b"SMPL" => ChunkType::Sampler,
        b"ltxt" | b"LTXT" | b"INST" | b"inst" => ChunkType::Instrument,
        b"acid" | b"ACID" => ChunkType::Acid,
        _ => ChunkType::Unknown(String::from_utf8_lossy(&tag).into_owned()),
    }
}

pub struct RiffFile {
    pub filename: String,
    pub chunks: Vec<RiffChunk>,
}

impl RiffFile {
    pub fn len(&self) -> usize {
        // (4 for WAVE header chunk, RIFF chunk not included)
        4 + self.chunks.iter().fold(0, |acc, &ref chunk| {
            acc + crate::utils::padded_size(chunk.len() as u32) as usize + 8
        }) // add 8 bytes for each chunks header
    }

    pub fn read(mut reader: fs::File, filename: String) -> Result<Self, io::Error> {
        // don't store stuff like the RIFF header chunk as it'll be regenerated on output
        {
            // read RIFF header
            let mut tag = [0u8; 4]; // header tag
            reader.read(&mut tag)?;

            if &tag != b"RIFF" {
                return Err(Error::new(ErrorKind::Other, "no RIFF tag found"));
            }
        }

        let _ = reader.read_u32::<LittleEndian>()?; // get file length (minus RIFF header).

        {
            // read WAVE header
            let mut tag = [0u8; 4]; // header tag
            reader.read(&mut tag)?;

            if &tag != b"WAVE" {
                return Err(Error::new(ErrorKind::Other, "no WAVE tag found"));
            }
        }

        let mut chunks = Vec::new();

        loop {
            // read chunks
            // let tag = reader.read_u32::<LittleEndian>()?;
            let mut tag = [0u8; 4]; // header tag

            let read_attempt = reader.read(&mut tag);

            let chunk_header = match read_attempt {
                Err(_) => {
                    println!("malformed RIFF file detected. skipping additional chunks.");
                    break;
                }
                Ok(header) => header,
            };

            if chunk_header == 0 {
                break; // end of file found
            }

            let chunk_len = match reader.read_u32::<LittleEndian>() {
                Err(_) => {
                    println!("malformed RIFF file detected. skipping additional chunks.");
                    break;
                }
                Ok(length) => length,
            };

            let chunk = Cursor::new(crate::utils::read_bytes(
                &mut reader,
                crate::utils::padded_size(chunk_len) as usize,
            )?);

            chunks.push(RiffChunk {
                data: chunk.into_inner(),
                header: header_to_rifftype(tag),
            });
        }

        Ok(RiffFile { filename, chunks })
    }

    pub fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    pub fn write(&self, mut writer: fs::File) -> Result<(), Error> {
        self.validate()?;

        // RIFF, WAVE, FMT, DATA chunks
        writer.write(b"RIFF")?; // RIFF tag
        writer.write_u32::<LittleEndian>(self.len() as u32)?; // file size (not including RIFF chunk of 8 bytes)
        writer.write(b"WAVE")?;

        for chunk in self.chunks.iter() {
            let header = chunk.header.clone();
            let chunk_len = chunk.len() as u32;

            writer.write(&header.to_tag())?;
            writer.write_u32::<LittleEndian>(chunk.len() as u32)?;

            if utils::padded_size(chunk_len) != chunk_len {
                // have to copy this chunk to mutably pad it :(
                let mut padded_chunk_data = chunk.data.clone();
                utils::pad_vec(
                    &mut padded_chunk_data,
                    (utils::padded_size(chunk_len) - chunk_len) as usize,
                );
                writer.write(&padded_chunk_data)?;
            } else {
                writer.write(&chunk.data)?;
            }
        }

        Ok(())
    }

    pub fn find_chunk_by_type(&self, chunktype: ChunkType) -> Option<&RiffChunk> {
        self.chunks.iter().find(|c| c.header == chunktype)
    }

    pub fn add_or_replace_chunk_by_type(&mut self, chunk: RiffChunk) {
        self.chunks.retain(|c| c.header != chunk.header);
        self.chunks.push(chunk);
    }
}
