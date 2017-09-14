use byteorder::{ WriteBytesExt, LittleEndian };

// use std::io;
use std::io::{ Write };
use std::fs::File;
use std::io::{ Error };

use WavFile;

impl WavFile {
    pub fn write(mut writer: File, wav: WavFile) -> Result<(), Error> {
        { // RIFF chunk
            writer.write(b"RIFF")?;                             // RIFF tag
            writer.write_u32::<LittleEndian>(wav.len())?;       // file size (not including RIFF chunk of 8 bytes)
            writer.write(b"WAVE")?;                             // RIFF type
        }

        { // FMT chunk
            writer.write(b"fmt ")?;                                         // tag
            writer.write_u32::<LittleEndian>(wav.format_chunk.len())?;      // chunk size (minus 8 bytes for header)
            writer.write(&wav.format_chunk.data)?;
        }

        { // DATA chunk
            writer.write(b"data")?;                                         // tag
            writer.write_u32::<LittleEndian>(wav.data_chunk.len())?;        // chunk size (minus 8 bytes for header)
            writer.write(&wav.data_chunk.data)?;
        }

        { // INST chunk
            match wav.instrument_chunk {
                Some(inst) => {
                    println!("Instrument chunk found, writing.");
                    writer.write(b"ltxt")?;                     // tag
                    writer.write_u32::<LittleEndian>(7)?;       // chunk size is always 7 for inst
                    writer.write(&inst.serialise())?;
                },
                None => { println!("Instrument chunk not found, skipping."); }
            }
        }

        { // CUE chunk
            if let Some(cue) = wav.cue_points.first() {
                let chunk = cue.serialise();

                println!("Writing: CUE");
                writer.write(b"cue ")?;                                     // tag
                writer.write_u32::<LittleEndian>(wav.cue_chunk_len())?;     // ChunkDataSize = 4 + (NumCuePoints * 24)
                writer.write_u32::<LittleEndian>(1_u32)?;                   // number of cue points

                writer.write(&chunk)?;
            } else { println!("Cue chunk not found, skipping."); }
        }

        Ok(())
    }
}
