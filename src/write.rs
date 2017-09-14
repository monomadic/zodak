use byteorder::{ WriteBytesExt, LittleEndian };

// use std::io;
use std::io::{ Write };
use std::fs::File;
use std::io::{ Error };

use WavFile;

impl WavFile {
    pub fn write(mut writer: File, wav: WavFile) -> Result<(), Error> {
        
        { // RIFF chunk
            println!("Writing: RIFF");
            writer.write(b"RIFF")?;                             // RIFF tag
            writer.write_u32::<LittleEndian>(wav.len())?;       // file size (not including RIFF chunk of 8 bytes)
            writer.write(b"WAVE")?;                             // RIFF type
        }

        { // FMT chunk
            println!("Writing: FMT");
            writer.write(b"fmt ")?;                                         // tag
            writer.write_u32::<LittleEndian>(wav.format_chunk.len())?;      // chunk size (minus 8 bytes for header)
            writer.write(&wav.format_chunk.data)?;
        }

        { // DATA chunk
            println!("Writing: DATA");
            writer.write(b"data")?;                                         // tag
            writer.write_u32::<LittleEndian>(wav.data_chunk.len())?;        // chunk size (minus 8 bytes for header)
            writer.write(&wav.data_chunk.data)?;
        }

        { // INST chunk
            match wav.instrument_chunk {
                Some(inst) => {
                    println!("Writing: INST");
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

        { // SMPL chunk
            match wav.sampler_chunk {
                Some(smpl) => {
                    println!("{:?}", smpl.serialise());
                    println!("Writing: SMPL");
                    writer.write(b"smpl")?;                         // tag
                    writer.write_u32::<LittleEndian>(smpl.len())?;  // chunk size
                    writer.write(&smpl.serialise())?;               // chunk data
                },
                None => { println!("Instrument chunk not found, skipping."); }
            }
        }

        Ok(())
    }
}
