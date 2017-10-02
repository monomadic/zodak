use byteorder::{ ReadBytesExt, LittleEndian };

use std::io;
use std::io::{ Cursor, Read, Error, ErrorKind };
use std::fs;

// pub trait RiffChunk {
//     fn header(&self) -> String;
//     fn len(&self) -> u32;
//     fn serialise(&self) -> Vec<u8>;
//     fn print(&self);
//     // fn structure(&self) -> Self;
// }

pub struct RiffChunk {
    pub header: ChunkType,
    data: Vec<u8>,
}

// impl RiffChunk {
//     pub fn get_type(&self) -> ChunkType {
//         ChunkType::Unknown
//     }
// }

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

pub struct RiffFile {
    pub filename: String,
    pub chunks: Vec<RiffChunk>,
}

impl RiffFile {
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
    
    pub fn read(mut reader: fs::File, filename: String) -> Result<Self, io::Error> {
        // don't store stuff like the RIFF header chunk as it'll be regenerated on output
        {   // read RIFF header
            let mut tag=[0u8;4]; // header tag
            reader.read(&mut tag)?;

            if &tag != b"RIFF" {
                return Err(Error::new(ErrorKind::Other, "no RIFF tag found"));
            }
        }

        // get file length (minus RIFF header).
        let file_len = reader.read_u32::<LittleEndian>()?;

        {   // read WAVE header 
            let mut tag=[0u8;4]; // header tag
            reader.read(&mut tag)?;

            if &tag != b"WAVE" {
                return Err(Error::new(ErrorKind::Other, "no WAVE tag found"));
            }
        }

        let mut chunks = Vec::new();

        loop { // read chunks
            // let tag = reader.read_u32::<LittleEndian>()?;
            let mut tag=[0u8;4]; // header tag
            let chunk_header_size = reader.read(&mut tag)?;
            if chunk_header_size == 0 {
                break; // end of file found
            }

            let chunk_len = reader.read_u32::<LittleEndian>()?; // size
            let mut chunk = Cursor::new(::read_bytes(&mut reader, chunk_len as usize)?);

            let mut data = chunk.into_inner();
            if ::padded_size(chunk_len) != chunk_len {

                println!("padding required for incorrect chunk size: {:?}, should be {:?}", chunk_len, ::padded_size(chunk_len));
                ::pad_vec(&mut data, (::padded_size(chunk_len) - chunk_len) as usize);

                println!("padding complete, new size: {:?}", data.len());
            }
            chunks.push(RiffChunk{ data: data, header: header_to_rifftype(tag) });
        }

        Ok(RiffFile {
            filename: filename,
            chunks: chunks
        })
    }

    pub fn find_chunk_by_type(&self, _: ChunkType) -> Option<RiffChunk> {
        None
    }
}

fn header_to_rifftype(tag: [u8;4]) -> ChunkType {
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
        _ => ChunkType::Unknown(format!("{:?}", tag)),
    }
}
