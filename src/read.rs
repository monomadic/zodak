use byteorder::{ ReadBytesExt, LittleEndian };

use std::io;
use std::io::{ Cursor, Read, Error, ErrorKind };
use std::fs::File;

use { WavFile, FormatChunk, DataChunk, InstrumentChunk, SamplerChunk, SampleLoop, LoopType, CuePoint };

impl WavFile {
    pub fn read(mut reader: File) -> Result<Self, io::Error> { // todo: change with BufReader

        let mut format_chunk: Option<FormatChunk> = None;
        let mut data_chunk: Option<DataChunk> = None;
        let mut instrument_chunk: Option<InstrumentChunk> = None;
        let mut sampler_chunk: Option<SamplerChunk> = None;
        let mut cue_points: Vec<CuePoint> = Vec::new();
        
        {   // read RIFF header
            let tag = reader.read_u32::<LittleEndian>()?;
            if &tag.to_string() == "RIFF" {
                return Err(Error::new(ErrorKind::Other, "no RIFF tag found"));
            } else { println!("Read: RIFF"); }
        }

        // get file length (minus RIFF header)
        let file_len = reader.read_u32::<LittleEndian>()?;
        println!("Filesize: {:?}", file_len);
        
        {   // read WAVE header 
            let tag = reader.read_u32::<LittleEndian>()?;
            if &tag.to_string() == "WAVE" {
                return Err(Error::new(ErrorKind::Other, "no WAVE tag found"));
            } else { println!("Read: WAVE"); }
        }

        loop { // read chunks
            // let tag = reader.read_u32::<LittleEndian>()?;
            let mut tag=[0u8;4]; // header tag
            let chunk_header_size = reader.read(&mut tag)?;
            if chunk_header_size == 0 {
                break; // end of file found
            }

            let chunk_len = reader.read_u32::<LittleEndian>()?; // size
            let mut chunk = Cursor::new(::read_bytes(&mut reader, chunk_len as usize)?);

            match &tag {
                b"fmt " => {
                    if chunk_len < 16 { return Err(Error::new(ErrorKind::Other, "invalid fmt chunk size")) };
                    format_chunk = Some(FormatChunk{
                        data: chunk.into_inner(),
                    });
                    println!("Read: FMT length: {:?}", chunk_len);
                },
                b"data" => {
                    data_chunk = Some(DataChunk{
                        data: chunk.into_inner(),
                    });
                    println!("Read: DATA length: {:?}", chunk_len);
                },
                b"fact" => {
                    println!("Read: FACT length: {:?}", chunk_len);
                },
                b"cue " => {
                    println!("Read: CUE length: {:?}", chunk_len);
                    let num_cue_points = chunk.read_u32::<LittleEndian>()?;
                    println!("  cue_points: {:?}", num_cue_points);
                    println!("  chunk: {:?}", chunk);

                    let chunk_data_size = 4 + (num_cue_points * 24); // 24 bytes per cue point (6 x u8)
                    if chunk_len < chunk_data_size { return Err(Error::new(ErrorKind::Other, "invalid cue chunk size")); }

                    cue_points.push(CuePoint {
                        id: chunk.read_u32::<LittleEndian>()?,
                        position: chunk.read_u32::<LittleEndian>()?,
                        data_chunk_id: chunk.read_u32::<LittleEndian>()?,
                        chunk_start: chunk.read_u32::<LittleEndian>()?,
                        block_start: chunk.read_u32::<LittleEndian>()?,
                        sample_offset: chunk.read_u32::<LittleEndian>()?,
                    });

                    // TODO: support for multiple cue points.

                    // println!("  {:?}", cue_point);
                    // println!("  data_chunk_id: {}", cue_point.data_chunk_id.to_string());
                },
                b"plst" => {
                    println!("Read: PLST length: {:?}", chunk_len);
                    let num_cue_points = chunk.read_u32::<LittleEndian>()?;
                    let chunk_data_size = num_cue_points * 12;
                    if chunk_len < chunk_data_size { return Err(Error::new(ErrorKind::Other, "invalid plst chunk size")) };
                },
                b"list" => {
                    println!("Read: LIST length: {:?}", chunk_len);

                },
                b"labl" => {
                    println!("Read: LABL length: {:?}", chunk_len);
                },
                // b"ltxt" => { println!("LTXT chunk found. length: {:?}", chunk_len); },
                b"note" => {
                    println!("Read: NOTE length: {:?}", chunk_len);
                },
                b"smpl" => {
                    println!("Read: SMPL length: {:?}", chunk_len);

                    sampler_chunk = Some(SamplerChunk {
                        manufacturer: chunk.read_u32::<LittleEndian>()?,
                        product: chunk.read_u32::<LittleEndian>()?,
                        sample_period: chunk.read_u32::<LittleEndian>()?,
                        midi_unity_note: chunk.read_u32::<LittleEndian>()?,
                        midi_pitch_fraction: chunk.read_u32::<LittleEndian>()?,
                        smpte_format: chunk.read_u32::<LittleEndian>()?,
                        smpte_offset: chunk.read_u32::<LittleEndian>()?,
                        sample_loops: {
                            let num_sample_loops = chunk.read_u32::<LittleEndian>()?;
                            let sampler_data_chunk_size = chunk.read_u32::<LittleEndian>()?;

                            (0..num_sample_loops).map(|i|
                                SampleLoop {
                                    id: chunk.read_u32::<LittleEndian>().unwrap(),
                                    loop_type: {
                                        let lt = chunk.read_u32::<LittleEndian>().unwrap();
                                        LoopType::Forward
                                    },
                                    start: chunk.read_u32::<LittleEndian>().unwrap(),
                                    end: chunk.read_u32::<LittleEndian>().unwrap(),
                                    fraction: chunk.read_u32::<LittleEndian>().unwrap(),
                                    play_count: chunk.read_u32::<LittleEndian>().unwrap(),
                                }
                            ).collect()
                        },
                        sampler_data: Vec::new(),
                    });

                    println!("  {:?}", sampler_chunk);
                    println!("  midi_unity_note: {}", ::note_num_to_name(sampler_chunk.clone().unwrap().midi_unity_note));
                },
                b"ltxt" => { // NOTE: 'inst' tag also works in ableton and is a possible replacement tag.
                    // The instrument chunk is used to describe how the waveform should be played as an instrument sound.
                    // This information is useful for communicating musical information between sample-based music programs,
                    // such as trackers or software wavetables. This chunk is optional and no more than 1 may appear in a
                    // WAVE file.
                    println!("Read: INST length: {:?}", chunk_len);

                    instrument_chunk = Some(InstrumentChunk {
                        unshifted_note: chunk.read_u8()?,
                        fine_tune: chunk.read_u8()?,
                        gain: chunk.read_u8()?,
                        low_note: chunk.read_u8()?,
                        high_note: chunk.read_u8()?,
                        low_vel: chunk.read_u8()?,
                        high_vel: chunk.read_u8()?,
                    });
                }, // this should be ltxt
                _ => { println!("WARNING: unknown chunk: {:?}, length: {:?}", ::std::str::from_utf8(&tag).unwrap(), chunk_len); }
            }
        }

        Ok(WavFile {
            format_chunk: format_chunk.unwrap(),
            data_chunk: data_chunk.unwrap(),
            sampler_chunk: sampler_chunk,
            instrument_chunk: instrument_chunk,
            cue_points: cue_points,
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