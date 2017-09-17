use byteorder::{ ByteOrder, WriteBytesExt, LittleEndian };

use std::io::{ Write };
use std::fs::File;
use std::io::{ Error };

use { WavFile, SamplerChunk, InstrumentChunk, CuePoint };
use padded_size; // implement

// note: should really build these up in memory first then flush to file, safer and easier to calculate sizes.
impl WavFile {
    pub fn write(mut writer: File, wav: WavFile) -> Result<(), Error> {

        // RIFF, WAVE, FMT, DATA chunks
        writer.write(b"RIFF")?;                             // RIFF tag
        writer.write_u32::<LittleEndian>(wav.len())?;       // file size (not including RIFF chunk of 8 bytes)
        writer.write(b"WAVE")?;                             // RIFF type
        writer.write_chunk(b"fmt ", wav.format_chunk.len(), &wav.format_chunk.data)?;
        writer.write_chunk(b"data", wav.data_chunk.len(), &wav.data_chunk.data)?;

        // INST chunk
        match wav.instrument_chunk {
            Some(inst) => {
                writer.write_chunk(b"inst", 7, &inst.serialise())?;
                // println!("Writing: INST, len: r:{} a:{}", 7, inst.serialise().len());
            },
            None => { println!("skipping: INST"); }
        }

        // CUE chunk
        if let Some(cue) = wav.cue_points.first() {
            let chunk = cue.serialise();

            // println!("Writing: CUE, len: r:{} a:{}", wav.cue_chunk_len(), chunk.len());
            writer.write(b"cue ")?;                                     // tag
            writer.write_u32::<LittleEndian>(wav.cue_chunk_len())?;     // ChunkDataSize = 4 + (NumCuePoints * 24)
            writer.write_u32::<LittleEndian>(1_u32)?;                   // number of cue points

            writer.write(&chunk)?;
        }

        // SMPL chunk
        match wav.sampler_chunk {
            Some(smpl) => {
                // println!("Writing: SMPL, len: r:{} a:{}", smpl.len(), smpl.serialise().len());
                writer.write(b"smpl")?;                         // tag
                writer.write_u32::<LittleEndian>(smpl.len())?;  // chunk size
                writer.write(&smpl.serialise())?;               // chunk data
            },
            None => { println!("skipping: SMPL"); }
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
            0   // zero padding to 8 bytes
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

trait ChunkWriter { fn write_chunk(&mut self, tag:&[u8; 4], size:u32, data:&Vec<u8>) -> Result<(), Error>; }
impl ChunkWriter for File {
    fn write_chunk(&mut self, tag:&[u8; 4], size:u32, data:&Vec<u8>) -> Result<(), Error> {
        // todo: check validity (correct file sizes)
        self.write(tag)?;                         // tag
        self.write_u32::<LittleEndian>(size)?;    // chunk size (minus 8 bytes for header)
        self.write(&data)?;                       // data
        Ok(())
    }
}

