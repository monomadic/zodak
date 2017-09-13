// https://sites.google.com/site/musicgapi/technical-documents/wav-file-format#inst

extern crate byteorder;

use byteorder::{ ByteOrder, ReadBytesExt, WriteBytesExt, LittleEndian };

use std::io;
use std::io::{ Read, Write };
// use std::io::prelude::*;

use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::Cursor;

mod write;

fn main() {
    let reader = File::open("resources/smpl_cue.wav").expect("input wav to read correctly.");
    let wav = WavFile::read(reader).expect("wav to parse correctly");

    let writer = File::create("output.wav").expect("output wav to create correctly.");
    let _ = WavFile::write(writer, wav);
}

struct WavFile {
    format_chunk: FormatChunk,
    data_chunk: DataChunk,
}

impl WavFile {
    pub fn len(&self) -> u32 {
        4 + // RIFF chunk
        self.format_chunk.len() + 8 +
        self.data_chunk.len() + 8
    }
}

struct FormatChunk {
    data: Vec<u8>,
}

impl FormatChunk {
    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

struct DataChunk {
    data: Vec<u8>,
}

impl DataChunk {
    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InstrumentChunk {
    /// The unshifted note field has the same meaning as the sampler chunk's MIDI Unity Note which specifies the
    /// musical note at which the sample will be played at it's original sample rate (the sample rate specified
    /// in the format chunk). (0-127)
    unshifted_note: u8,
    fine_tune: u8,
    gain: u8,
    low_note: u8,
    high_note: u8,
    low_vel: u8,
    high_vel: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CuePoint {
    id: u32,
    position: u32,
    data_chunk_id: u32,
    chunk_start: u32,
    block_start: u32,
    sample_offset: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SamplerChunk {

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
    manufacturer: u32,

    /// The product field specifies the MIDI model ID defined by the manufacturer corresponding to
    /// the Manufacturer field. Contact the manufacturer of the sampler to get the model ID. If no
    /// particular manufacturer's product is to be specified, a value of 0 should be used.
    product: u32,

    sample_period: u32,

    midi_unity_note: u32,

    midi_pitch_fraction: u32,

    smpte_format: u32,

    smpte_offset: u32,

    num_sample_loops: u32,

    sampler_data: u32,

}

impl WavFile {
    pub fn read(mut reader: File) -> Result<Self, io::Error> { // todo: change with BufReader

        let mut format_chunk: Option<FormatChunk> = None;
        let mut data_chunk: Option<DataChunk> = None;
        
        {   // read RIFF header
            let tag = reader.read_u32::<LittleEndian>()?;
            if &tag.to_string() == "RIFF" {
                return Err(Error::new(ErrorKind::Other, "no RIFF tag found"));
            } else { println!("RIFF tag found."); }
        }

        // get file length (minus RIFF header)
        let file_len = reader.read_u32::<LittleEndian>()?;
        println!("Filesize: {:?}", file_len);
        
        {   // read WAVE header 
            let tag = reader.read_u32::<LittleEndian>()?;
            if &tag.to_string() == "WAVE" {
                return Err(Error::new(ErrorKind::Other, "no WAVE tag found"));
            } else { println!("WAVE tag found."); }
        }

        loop { // read chunks
            // let tag = reader.read_u32::<LittleEndian>()?;
            let mut tag=[0u8;4]; // header tag
            let chunk_header_size = reader.read(&mut tag)?;
            if chunk_header_size == 0 {
                break; // end of file found
            }

            let chunk_len = reader.read_u32::<LittleEndian>()?; // size
            let mut chunk = Cursor::new(read_bytes(&mut reader, chunk_len as usize)?);

            match &tag {
                b"fmt " => {
                    println!("FMT  chunk found. length: {:?}", chunk_len);
                    if chunk_len < 16 { return Err(Error::new(ErrorKind::Other, "invalid fmt chunk size")) };
                    format_chunk = Some(FormatChunk{
                        data: chunk.into_inner(),
                    });
                },
                b"data" => {
                    println!("DATA chunk found. length: {:?}", chunk_len);
                    data_chunk = Some(DataChunk{
                        data: chunk.into_inner(),
                    });
                },
                b"fact" => {
                    println!("FACT chunk found. length: {:?}", chunk_len);
                },
                b"cue " => {
                    println!("CUE  chunk found. length: {:?}", chunk_len);
                    let num_cue_points = chunk.read_u32::<LittleEndian>()?;
                    println!("  cue points: {:?}", num_cue_points);

                    let chunk_data_size = 4 + (num_cue_points * 24); // 24 bytes per cue point (6 x u8)
                    if chunk_len < chunk_data_size { return Err(Error::new(ErrorKind::Other, "invalid cue chunk size")); }

                    let cue_point = CuePoint {
                        id: chunk.read_u32::<LittleEndian>()?,
                        position: chunk.read_u32::<LittleEndian>()?,
                        data_chunk_id: chunk.read_u32::<LittleEndian>()?,
                        chunk_start: chunk.read_u32::<LittleEndian>()?,
                        block_start: chunk.read_u32::<LittleEndian>()?,
                        sample_offset: chunk.read_u32::<LittleEndian>()?,
                    };

                    println!("  {:?}", cue_point);
                    println!("  data_chunk_id: {}", cue_point.data_chunk_id.to_string());
                },
                b"plst" => {
                    println!("PLST chunk found. length: {:?}", chunk_len);
                    let num_cue_points = chunk.read_u32::<LittleEndian>()?;
                    let chunk_data_size = num_cue_points * 12;
                    if chunk_len < chunk_data_size { return Err(Error::new(ErrorKind::Other, "invalid plst chunk size")) };
                },
                b"list" => {
                    println!("LIST chunk found. length: {:?}", chunk_len);

                },
                b"labl" => {
                    println!("LABL chunk found. length: {:?}", chunk_len);
                },
                // b"ltxt" => { println!("LTXT chunk found. length: {:?}", chunk_len); },
                b"note" => {
                    println!("NOTE chunk found. length: {:?}", chunk_len);
                },
                b"smpl" => {
                    println!("SMPL chunk found. length: {:?}", chunk_len);

                    let sampler_chunk = SamplerChunk {
                        manufacturer: chunk.read_u32::<LittleEndian>()?,
                        product: chunk.read_u32::<LittleEndian>()?,
                        sample_period: chunk.read_u32::<LittleEndian>()?,
                        midi_unity_note: chunk.read_u32::<LittleEndian>()?,
                        midi_pitch_fraction: chunk.read_u32::<LittleEndian>()?,
                        smpte_format: chunk.read_u32::<LittleEndian>()?,
                        smpte_offset: chunk.read_u32::<LittleEndian>()?,
                        num_sample_loops: chunk.read_u32::<LittleEndian>()?,
                        sampler_data: chunk.read_u32::<LittleEndian>()?,
                    };

                    println!("  {:?}", sampler_chunk);
                    println!("  midi_unity_note: {}", note_num_to_name(sampler_chunk.midi_unity_note));
                },
                b"ltxt" => {
                    // The instrument chunk is used to describe how the waveform should be played as an instrument sound.
                    // This information is useful for communicating musical information between sample-based music programs,
                    // such as trackers or software wavetables. This chunk is optional and no more than 1 may appear in a
                    // WAVE file.
                    println!("INST chunk found. length: {:?}", chunk_len);

                    let instrument_chunk = InstrumentChunk {
                        unshifted_note: chunk.read_u8()?,
                        fine_tune: chunk.read_u8()?,
                        gain: chunk.read_u8()?,
                        low_note: chunk.read_u8()?,
                        high_note: chunk.read_u8()?,
                        low_vel: chunk.read_u8()?,
                        high_vel: chunk.read_u8()?,
                    };

                    println!("  {:?}", instrument_chunk);

                }, // this should be ltxt
                _ => { println!("WARNING: unknown chunk: {:?}, length: {:?}", std::str::from_utf8(&tag).unwrap(), chunk_len); }
            }

            println!("");
        }

        Ok(WavFile {
            format_chunk: format_chunk.unwrap(),
            data_chunk: data_chunk.unwrap(),
        })
    }

    // fn read_chunks(mut reader: File) -> Result<(), Error> {
    //     let mut spec_opt = None;

    //     loop {
    //         let header = try!(WavReader::read_chunk_header(&mut reader));
    //         match header.kind {
    //             ChunkKind::Fmt => {
    //                 let spec = try!(WavReader::read_fmt_chunk(&mut reader, header.len));
    //                 spec_opt = Some(spec);
    //             }
    //             ChunkKind::Fact => {
    //                 // All (compressed) non-PCM formats must have a fact chunk
    //                 // (Rev. 3 documentation). The chunk contains at least one
    //                 // value, the number of samples in the file.
    //                 //
    //                 // The number of samples field is redundant for sampled
    //                 // data, since the Data chunk indicates the length of the
    //                 // data. The number of samples can be determined from the
    //                 // length of the data and the container size as determined
    //                 // from the Format chunk.
    //                 // http://www-mmsp.ece.mcgill.ca/documents/audioformats/wave/wave.html
    //                 let _samples_per_channel = reader.read_le_u32();
    //             }
    //             ChunkKind::Data => {
    //                 // The "fmt" chunk must precede the "data" chunk. Any
    //                 // chunks that come after the data chunk will be ignored.
    //                 if let Some(spec) = spec_opt {
    //                     return Ok((spec, header.len));
    //                 } else {
    //                     return Err(Error::FormatError("missing fmt chunk"));
    //                 }
    //             }
    //             ChunkKind::Unknown => {
    //                 // Ignore the chunk; skip all of its bytes.
    //                 try!(reader.skip_bytes(header.len as usize));
    //             }
    //         }
    //         // If no data chunk is ever encountered, the function will return
    //         // via one of the try! macros that return an Err on end of file.
    //     }
    // }
     
    // fn read_header(&mut self) -> Result<(), io::Error> {

    //     #[repr(C, packed)]
    //     #[derive(Debug)]
    //     struct RIFFChunk {
    //         header: [u8;4],
    //         size: [u8;4],
    //     }

    //     let num_bytes = ::std::mem::size_of::<RIFFChunk>();

    //     Ok(file_len)
    // }
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
    let oct = (num as f32 /12 as f32).floor()-1.0;
    let nmt = ((num%12)*2) as usize;
    let slice =
        if NSTRS.as_bytes()[nmt+1] == ' ' as u8{
            &NSTRS[nmt..(nmt+1)]
        } else {
            &NSTRS[nmt..(nmt+2)]
        };
    format!("{}{}",slice,oct)
}
