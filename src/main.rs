// https://sites.google.com/site/musicgapi/technical-documents/wav-file-format#inst

extern crate wavedit;

use wavedit::{ WavFile, InstrumentChunk };

use std::io;
use std::io::{ Cursor, Read, Error, ErrorKind };
use std::fs::File;

fn main() {
    let reader = File::open("resources/smpl_cue.wav").expect("input wav to read correctly.");
    let mut wav = WavFile::read(reader).expect("wav to parse correctly");

    wav.instrument_chunk = Some(InstrumentChunk {
        unshifted_note: 45,
        fine_tune: 0,
        gain: 0,
        low_note: 40,
        high_note: 50,
        low_vel: 10,
        high_vel: 120,
    });

    let writer = File::create("output.wav").expect("output wav to create correctly.");
    let _ = WavFile::write(writer, wav);
}
