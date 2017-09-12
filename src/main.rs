extern crate byteorder;

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};

use std::io;
use std::io::Read;
use std::io::prelude::*;
use std::fs::File;
use std::io::{Error, ErrorKind};

fn main() {
    read_wav();
}

fn read_wav() -> Result<(), io::Error> {
    let mut reader = File::open("resources/Dagga-A2.wav")?;

    let mut wav = WavFile::new(reader);

    Ok(())
}

struct WavFile {
    reader: File,
    file_len: u32,
    // format_chunk: FormatChunk,

}

impl WavFile {
    pub fn new(mut reader: File) -> Result<Self, io::Error> { // todo: change with BufReader
        
        {   // read RIFF header
            let mut buf=[0u8;4];
            reader.read(&mut buf)?;

            // let mut hdr = reader.read_uint::<LittleEndian>(4)?;

            if b"RIFF" != &buf {
                return Err(Error::new(ErrorKind::Other, "no RIFF tag found"));
            } else { println!("RIFF tag found."); }
        }

        // let file_len = try!(read_le_u32(&mut reader));
        let file_len = reader.read_u32::<LittleEndian>()?;
        println!("Filesize: {:?}", file_len);
        
        {   // read WAVE header 
            let mut buf=[0u8;4];
            reader.read(&mut buf)?;

            if b"WAVE" != &buf {
                return Err(Error::new(ErrorKind::Other, "no WAVE tag found"));
            } else { println!("WAVE tag found."); }
        }

        loop { // read chunks
            let mut buf=[0u8;4];
            reader.read(&mut buf)?;
            let chunk_len = reader.read_u32::<LittleEndian>()?;

            match &buf {
                b"fmt " => {
                    println!("FMT  chunk found. length: {:?}", chunk_len);
                    if chunk_len < 16 { return Err(Error::new(ErrorKind::Other, "invalid fmt chunk size")); }

                    let _ = read_bytes(&mut reader, chunk_len as usize)?;
                },
                b"data" => { println!("DATA chunk found. length: {:?}", chunk_len); },
                b"fact" => { println!("FACT chunk found. length: {:?}", chunk_len); },
                b"cue " => { println!("CUE  chunk found. length: {:?}", chunk_len); },
                b"plst" => { println!("PLST chunk found. length: {:?}", chunk_len); },
                b"list" => { println!("LIST chunk found. length: {:?}", chunk_len); },
                b"labl" => { println!("LABL chunk found. length: {:?}", chunk_len); },
                b"ltxt" => { println!("LTXT chunk found. length: {:?}", chunk_len); },
                b"note" => { println!("NOTE chunk found. length: {:?}", chunk_len); },
                b"smpl" => { println!("SMPL chunk found. length: {:?}", chunk_len); },
                b"inst" => { println!("INST chunk found. length: {:?}", chunk_len); },
                _ => { }
            }
        }

        Ok(WavFile {
            reader: reader,
            file_len: file_len,
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
    let mut buf = Vec::with_capacity(n);
    unsafe { buf.set_len(n); }
    try!(io::copy(&mut reader.take(n as u64), &mut buf));
    Ok(buf)
}

// fn read_exact(&mut self, n: u64) -> io::Result<Vec<u8>> {
//         let mut buf = vec![];
//         try!(io::copy(&mut self.take(n), &mut buf));
//         Ok(buf)
//     }
// fn read_le_u32(mut file: &mut File) -> io::Result<u32> {
//     let mut buf = [0u8; 4];
//     try!(file.read(&mut buf));
//     Ok((buf[3] as u32) << 24 | (buf[2] as u32) << 16 |
//        (buf[1] as u32) << 8  | (buf[0] as u32) << 0)
// }