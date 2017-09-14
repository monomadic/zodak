use byteorder::{ ByteOrder, WriteBytesExt, LittleEndian };

use std::io::{ Write };
use std::fs::File;
use std::io::{ Error };

use { WavFile, SamplerChunk, InstrumentChunk, CuePoint };

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
                None => { println!("Missing: INST"); }
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
            } else { println!("Missing: CUE"); }
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
                None => { println!("Missing: SMPL"); }
            }
        }

        Ok(())
    }
}

impl SamplerChunk {
    pub fn serialise(&self) -> Vec<u8> {
        // let num_sample_loops = self.sample_loops.len() as u32;
        // let size_of_data_chunk = num_sample_loops * 24;

        let mut chunk = Vec::with_capacity(36 + 24); // space for static fields and sample_loops
        unsafe { chunk.set_len(36 + 24) }; // todo: find a safe way to zero the elements.

        let sample_loop = self.sample_loops.first().unwrap();
        let sampler_data = 0; // greater than 0 if extra sampler data is present.

        LittleEndian::write_u32_into(&vec![
            self.manufacturer, self.product, self.sample_period, self.midi_unity_note, self.midi_pitch_fraction,
            self.smpte_format, self.smpte_offset, 1_u32, 0,
            sample_loop.id, 0_u32, sample_loop.start, sample_loop.end, sample_loop.fraction, sample_loop.play_count,
        ], &mut chunk);

        chunk
    }
}

impl InstrumentChunk {
    pub fn serialise(&self) -> Vec<u8> {
        vec![
            self.unshifted_note,
            self.fine_tune,
            self.gain,
            self.low_note,
            self.high_note,
            self.low_vel,
            self.high_vel,
        ]
    }
}

impl CuePoint {
    pub fn serialise(&self) -> Vec<u8> {
        let mut chunk = Vec::with_capacity(24);
        unsafe { chunk.set_len(24) }; // todo: find a safe way to zero the elements.

        LittleEndian::write_u32_into(&vec![
            self.id, self.position, self.data_chunk_id, self.chunk_start, self.block_start, self.sample_offset
        ], &mut chunk);

        chunk
    }
}
