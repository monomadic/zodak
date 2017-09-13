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

        Ok(())
    }
}