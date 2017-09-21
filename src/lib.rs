// https://sites.google.com/site/musicgapi/technical-documents/wav-file-format#inst

extern crate byteorder;
use byteorder::{ ByteOrder, LittleEndian };

use std::io;
use std::io::{ Cursor, Read, Error, ErrorKind };
use std::fs::File;

mod write;
pub use write::*;
mod read;
pub use read::*;

pub trait RiffChunk {
    fn header(&self) -> String;
    fn len(&self) -> u32;
    fn serialise(&self) -> Vec<u8>;
    fn print(&self);
}

pub struct WavFile {
    pub format_chunk: FormatChunk,
    pub data_chunk: DataChunk,
    pub sampler_chunk: Option<SamplerChunk>,

    /// The instrument chunk is used to describe how the waveform should be played as an instrument sound.
    /// This information is useful for communicating musical information between sample-based music programs,
    /// such as trackers or software wavetables. This chunk is optional and no more than 1 may appear in a
    /// WAVE file.
    pub instrument_chunk: Option<InstrumentChunk>,
    pub cue_points: Vec<CuePoint>,
}

impl WavFile {
    pub fn len(&self) -> u32 {
        let sampler_chunk_len: u32 = match self.sampler_chunk {
            Some(ref s) => { s.len() + 8 },
            _ => 0,
        };

        let instrument_chunk_len: u32 = match self.sampler_chunk {
            Some(_) => { 7 + 8 },
            _ => 0,
        };

        4 + // RIFF chunk
        self.format_chunk.len() + 8 +
        self.data_chunk.len() + 8 +
        sampler_chunk_len + 
        instrument_chunk_len +
        self.cue_chunk_len()
    }

    pub fn cue_chunk_len(&self) -> u32 {
        if self.cue_points.is_empty() {
            0
        } else {
            4_u32 + (self.cue_points.len() as u32 * 24)
        }
    }

    pub fn file_name(&self, name:&str) -> String {
        match self.sampler_chunk {
            Some(ref s) => {
                let note_name = ::note_num_to_name(s.midi_unity_note);
                format!("{} {}.wav", name.trim(), note_name)
            },
            None => format!("{}.wav", name.trim())
        }
    }

    pub fn pitch_keycenter(&self) -> Result<String, Error> {
        match self.sampler_chunk {
            Some(ref s) => Ok(::note_num_to_name(s.midi_unity_note)),
            None => Err(Error::new(ErrorKind::Other, "called pitch_keycenter when sampler_chunk is not present"))
        }
    }

    pub fn key_range(&self) -> Result<(String, String), Error> {
        match self.instrument_chunk {
            Some(ref i) => Ok(
                ( ::note_num_to_name(i.low_note as u32), ::note_num_to_name(i.high_note as u32) )
            ),
            None => Err(Error::new(ErrorKind::Other, "called key_range when instrument_chunk is not present"))
        }
    }

    pub fn loop_points(&self) -> Result<(u32, u32), Error> {
        match self.sampler_chunk {
            Some(ref s) => {
                match s.sample_loops.first() {
                    Some(l) => Ok(( l.start, l.end )),
                    None => Err(Error::new(ErrorKind::Other, "called loop_points when sampler_chunk has no loops"))
                }
            },
            None => Err(Error::new(ErrorKind::Other, "called loop_points when sampler_chunk is not present"))
        }
    }
}

pub struct FormatChunk {
    data: Vec<u8>,
}

impl FormatChunk {
    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

pub struct DataChunk {
    data: Vec<u8>,
}

impl DataChunk {
    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstrumentChunk {
    /// The unshifted note field has the same meaning as the sampler chunk's MIDI Unity Note which specifies the
    /// musical note at which the sample will be played at it's original sample rate (the sample rate specified
    /// in the format chunk). (0-127)
    pub unshifted_note: u8,

    /// Fine Tune (dB)
    /// The fine tune value specifies how much the sample's pitch should be altered when the sound is played back
    /// in cents (1/100 of a semitone). A negative value means that the pitch should be played lower and a positive
    /// value means that it should be played at a higher pitch.
    pub fine_tune: u8, // -50 - +50

    /// The gain value specifies the number of decibels to adjust the output when it is played. A value of 0dB
    /// means no change, 6dB means double the amplitude of each sample and -6dB means to halve the amplitude of
    /// each sample. Every additional +/-6dB will double or halve the amplitude again.
    pub gain: u8, // -64 - +64
    pub low_note: u8,
    pub high_note: u8,
    pub low_vel: u8,
    pub high_vel: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CuePoint {
    /// ID
    /// Each cue point has a unique identification value used to associate cue points with information in other
    /// chunks. For example, a Label chunk contains text that describes a point in the wave file by referencing
    /// the associated cue point.
    id: u32,

    /// Position
    /// The position specifies the sample offset associated with the cue point in terms of the sample's position
    /// in the final stream of samples generated by the play list. Said in another way, if a play list chunk is
    /// specified, the position value is equal to the sample number at which this cue point will occur during
    /// playback of the entire play list as defined by the play list's order. If no play list chunk is specified
    /// this value should be 0.
    position: u32,

    /// Data Chunk ID
    /// This value specifies the four byte ID used by the chunk containing the sample that corresponds to this cue
    /// point. A Wave file with no play list is always "data". A Wave file with a play list containing both sample
    /// data and silence may be either "data" or "slnt".
    data_chunk_id: u32,

    /// Chunk Start
    /// The Chunk Start value specifies the byte offset into the Wave List Chunk of the chunk containing the sample
    /// that corresponds to this cue point. This is the same chunk described by the Data Chunk ID value. If no Wave
    /// List Chunk exists in the Wave file, this value is 0. If a Wave List Chunk exists, this is the offset into
    /// the "wavl" chunk. The first chunk in the Wave List Chunk would be specified with a value of 0.
    chunk_start: u32,

    /// Block Start
    /// The Block Start value specifies the byte offset into the "data" or "slnt" Chunk to the start of the block
    /// containing the sample. The start of a block is defined as the first byte in uncompressed PCM wave data or
    /// the last byte in compressed wave data where decompression can begin to find the value of the corresponding
    /// sample value.
    block_start: u32,

    /// Sample Offset
    /// The Sample Offset specifies an offset into the block (specified by Block Start) for the sample that
    /// corresponds to the cue point. In uncompressed PCM waveform data, this is simply the byte offset into the
    /// "data" chunk. In compressed waveform data, this value is equal to the number of samples (may or may not be bytes)
    /// from the Block Start to the sample that corresponds to the cue point.
    sample_offset: u32,
}

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

    pub sample_period: u32,

    pub midi_unity_note: u32,

    pub midi_pitch_fraction: u32,

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
    pub id: u32,

    /// The type field defines how the waveform samples will be looped.
    pub loop_type: LoopType,
    pub start: u32,
    pub end: u32,
    pub fraction: u32,
    pub play_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoopType {
    Forward,
    PingPong,
    Reverse,
}

impl SamplerChunk {
    pub fn len(&self) -> u32 {
        36 + (self.sample_loops.len() as u32 * 24) + self.sampler_data.len() as u32
    }
}

fn read_bytes(ref mut reader: &mut File, n: usize) -> io::Result<Vec<u8>> {
    // We allocate a runtime fixed size buffer, and we are going to read
    // into it, so zeroing or filling the buffer is a waste. This method
    // is safe, because the contents of the buffer are only exposed when
    // they have been overwritten completely by the read.
    // let mut buf = Vec::with_capacity(n);
    // unsafe { buf.set_len(n); }

    let mut buf = vec![];
    try!(io::copy(&mut reader.take(n as u64), &mut buf));

    Ok(buf)
}

static NSTRS: &'static str = "C C#D D#E F F#G G#A A#B ";

/// convert a midi note number to a name
pub fn note_num_to_name(num: u32) -> String {
    let oct = (num as f32 /12 as f32).floor()-2.0;
    let nmt = ((num%12)*2) as usize;
    let slice =
        if NSTRS.as_bytes()[nmt+1] == ' ' as u8{
            &NSTRS[nmt..(nmt+1)]
        } else {
            &NSTRS[nmt..(nmt+2)]
        };
    format!("{}{}",slice,oct)
}

pub fn name_to_note_num(note_name:&str) -> u8 {
    use std::collections::HashMap;
    let mut notes:HashMap<&str,u8> = HashMap::new();
    notes.insert("c", 0);
    notes.insert("d", 2);
    notes.insert("e", 4);
    notes.insert("f", 5);
    notes.insert("g", 7);
    notes.insert("a", 9);
    notes.insert("b", 11);

    let note_len = note_name.len();
    let octave:Vec<char> = note_name.chars().skip(note_len - 1).take(1).collect();
    let octave:u8 = octave[0].to_string().parse().expect("octave to be converted from a digit string");
    let note:Vec<char> = note_name.chars().take(note_len - 1).collect();


    let base_note = *notes.get(&note[0].clone().to_lowercase().to_string().as_str()).expect("note to be a valid midi note");

    let octave = octave;
    let mut midi_note = base_note + (octave * 12);

    if note.iter().find(|&&x| x == '#' ).is_some() { midi_note = midi_note + 1; }
    if note.iter().find(|&&x| x == '-' ).is_some() { midi_note = midi_note - (24 * octave); }

    midi_note + 24
}

pub fn padded_size(size: u32) -> u32 {
    (((size + 1) / 2) * 2)
}

pub fn pad_vec(mut v:&mut Vec<u8>, size: usize) {
    println!("attempting to pad {} bytes.", size);
    for _ in 0..size { print!(":"); v.push(0) };
}