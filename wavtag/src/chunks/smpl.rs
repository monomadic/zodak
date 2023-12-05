use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};

use std::io;
use std::io::{Cursor, Error, ErrorKind};

use crate::{ChunkType, RiffChunk, RiffFile};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SamplerChunk {
    /// The manufacturer field specifies the MIDI Manufacturer's Association (MMA) Manufacturer
    /// code for the sampler intended to receive this file's waveform. Each manufacturer of a
    /// MIDI product is assigned a unique ID which identifies the company. If no particular
    /// manufacturer is to be specified, a value of 0 should be used.
    ///
    /// The value is stored with some extra information to enable translation to the value used in
    /// a MIDI System Exclusive transmission to the sampler. The high byte indicates the number of
    /// low order bytes (1 or 3) that are valid for the manufacturer code. For example, the value
    /// for Digidesign will be 0x01000013 (0x13) and the value for Microsoft will be 0x30000041
    /// (0x00, 0x00, 0x41). See the MIDI Manufacturers List for a list.
    pub manufacturer: u32,

    /// The product field specifies the MIDI model ID defined by the manufacturer corresponding to
    /// the Manufacturer field. Contact the manufacturer of the sampler to get the model ID. If no
    /// particular manufacturer's product is to be specified, a value of 0 should be used.
    pub product: u32,

    /// The sample period specifies the duration of time that passes during the playback of one sample
    /// in nanoseconds (normally equal to 1 / Samplers Per Second, where Samples Per Second is the value
    /// found in the format chunk).
    pub sample_period: u32,

    /// The MIDI unity note value has the same meaning as the instrument chunk's MIDI Unshifted Note field
    /// which specifies the musical note at which the sample will be played at it's original sample rate
    /// (the sample rate specified in the format chunk).
    pub midi_unity_note: u32,

    /// The MIDI pitch fraction specifies the fraction of a semitone up from the specified MIDI unity note
    /// field. A value of 0x80000000 means 1/2 semitone (50 cents) and a value of 0x00000000 means no fine
    /// tuning between semitones.
    pub midi_pitch_fraction: u32,

    /// The SMPTE format specifies the Society of Motion Pictures and Television E time format used in the
    /// following SMPTE Offset field. If a value of 0 is set, SMPTE Offset should also be set to 0.
    pub smpte_format: u32,

    pub smpte_offset: u32,

    pub sample_loops: Vec<SampleLoop>,

    /// Sampler Data
    /// The sampler data value specifies the number of bytes that will follow this chunk (including
    /// the entire sample loop list). This value is greater than 0 when an application needs to save
    /// additional information. This value is reflected in this chunks data size value.
    pub sampler_data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleLoop {
    /// The Cue Point ID specifies the unique ID that corresponds to one of the defined cue points
    /// in the cue point list. Furthermore, this ID corresponds to any labels defined in the
    /// associated data list chunk which allows text labels to be assigned to the various sample loops.
    pub id: u32,

    /// The type field defines how the waveform samples will be looped.
    /// 0 - Loop forward (normal)
    /// 1 - Alternating loop (forward/backward, also known as Ping Pong)
    /// 2 - Loop backward (reverse)
    /// 3 - 31 - Reserved for future standard types
    /// 32 - 0xFFFFFFFF - Sampler specific types (defined by manufacturer)
    pub loop_type: LoopType,

    pub start: u32,
    pub end: u32,
    pub fraction: u32,

    /// The play count value determines the number of times to play the loop.
    /// A value of 0 specifies an infinite sustain loop. An infinite sustain
    /// loop will continue looping until some external force interrupts playback,
    /// such as the musician releasing the key that triggered the wave's playback.
    /// All other values specify an absolute number of times to loop.
    pub play_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoopType {
    Forward,
    PingPong,
    Reverse,
}

impl Default for SamplerChunk {
    fn default() -> Self {
        Self {
            manufacturer: 0,
            product: 0,
            sample_period: 0,
            midi_unity_note: 0,
            midi_pitch_fraction: 0,
            smpte_format: 0,
            smpte_offset: 0,
            sample_loops: Vec::new(),
            sampler_data: Vec::new(),
        }
    }
}

impl SamplerChunk {
    pub fn from_chunk(chunk: &RiffChunk) -> Result<Self, io::Error> {
        if chunk.header != ChunkType::Sampler {
            return Err(Error::new(
                ErrorKind::Other,
                "attempted from_chunk() on non-sampler chunk",
            ));
        };

        let mut data = Cursor::new(&chunk.data);

        Ok(SamplerChunk {
            manufacturer: data.read_u32::<LittleEndian>()?,
            product: data.read_u32::<LittleEndian>()?,
            sample_period: data.read_u32::<LittleEndian>()?,
            midi_unity_note: data.read_u32::<LittleEndian>()?,
            midi_pitch_fraction: data.read_u32::<LittleEndian>()?,
            smpte_format: data.read_u32::<LittleEndian>()?,
            smpte_offset: data.read_u32::<LittleEndian>()?,
            sample_loops: {
                let num_sample_loops = data.read_u32::<LittleEndian>()?;
                let _ = data.read_u32::<LittleEndian>()?; // sampler_data_chunk_size

                (0..num_sample_loops)
                    .map(|_| SampleLoop {
                        id: data.read_u32::<LittleEndian>().unwrap(),
                        loop_type: {
                            let lt = data.read_u32::<LittleEndian>().unwrap();
                            match lt {
                                // TODO: other loop types!
                                _ => LoopType::Forward,
                            }
                        },
                        start: data.read_u32::<LittleEndian>().unwrap(),
                        end: data.read_u32::<LittleEndian>().unwrap(),
                        fraction: data.read_u32::<LittleEndian>().unwrap(),
                        play_count: data.read_u32::<LittleEndian>().unwrap(),
                    })
                    .collect()
            },
            sampler_data: Vec::new(),
        })
    }

    pub fn serialise(&self) -> Vec<u8> {
        let mut chunk = Vec::with_capacity(36 + 24); // space for static fields and sample_loops
        unsafe { chunk.set_len(36 + 24) }; // todo: find a safe way to zero the elements.

        let sample_loop = self
            .sample_loops
            .first()
            .expect("sampler chunk to have at least one sample loop");
        let _ = 0; // sampler_data: greater than 0 if extra sampler data is present.

        LittleEndian::write_u32_into(
            &vec![
                self.manufacturer,
                self.product,
                self.sample_period,
                self.midi_unity_note,
                self.midi_pitch_fraction,
                self.smpte_format,
                self.smpte_offset,
                1_u32,
                0,
                sample_loop.id,
                0_u32,
                sample_loop.start,
                sample_loop.end,
                sample_loop.fraction,
                sample_loop.play_count,
            ],
            &mut chunk,
        );

        chunk
    }
}

impl RiffFile {
    pub fn get_sampler_chunk(&self) -> SamplerChunk {
        match self.find_chunk_by_type(ChunkType::Sampler) {
            Some(c) => SamplerChunk::from_chunk(c).expect("chunk to be a valid sampler chunk"),
            None => SamplerChunk::default(),
        }
    }

    pub fn set_sampler_chunk(&mut self, chunk: SamplerChunk) {
        self.add_or_replace_chunk_by_type(RiffChunk {
            header: ChunkType::Sampler,
            data: chunk.serialise(),
        });
    }
}
