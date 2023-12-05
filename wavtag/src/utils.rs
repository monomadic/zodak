use std::fs::File;
use std::io;
use std::io::Read;

pub fn read_bytes(ref mut reader: &mut File, n: usize) -> io::Result<Vec<u8>> {
    // We allocate a runtime fixed size buffer, and we are going to read
    // into it, so zeroing or filling the buffer is a waste. This method
    // is safe, because the contents of the buffer are only exposed when
    // they have been overwritten completely by the read.
    // let mut buf = Vec::with_capacity(n);
    // unsafe { buf.set_len(n); }

    let mut buf = vec![];
    io::copy(&mut reader.take(n as u64), &mut buf).expect("file to have enough bytes to read");

    Ok(buf)
}

static NSTRS: &'static str = "C C#D D#E F F#G G#A A#B ";

/// convert a midi note number to a name
pub fn note_num_to_name(num: u32) -> String {
    let oct = (num as f32 / 12 as f32).floor() - 2.0;
    let nmt = ((num % 12) * 2) as usize;
    let slice = if NSTRS.as_bytes()[nmt + 1] == ' ' as u8 {
        &NSTRS[nmt..(nmt + 1)]
    } else {
        &NSTRS[nmt..(nmt + 2)]
    };
    format!("{}{}", slice, oct)
}

pub fn padded_size(size: u32) -> u32 {
    ((size + 1) / 2) * 2
}

pub fn pad_vec(v: &mut Vec<u8>, size: usize) {
    // println!("attempting to pad {} bytes.", size);
    for _ in 0..size {
        v.push(0)
    }
}

pub fn str_to_int(str: &str) -> u32 {
    str.trim().parse().unwrap()
}

pub fn dir_as_string(path: &str) -> String {
    use std::path::PathBuf;
    let mut current_dir = PathBuf::new();
    if path == "." {
        use std::env;
        current_dir = env::current_dir().unwrap();
    } else {
        current_dir.push(path);
    }
    current_dir
        .file_name()
        .expect("current directory to be valid")
        .to_string_lossy()
        .into_owned()
}

