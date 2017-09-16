// https://sites.google.com/site/musicgapi/technical-documents/wav-file-format#inst

extern crate wavedit;
// extern crate glob;
extern crate docopt;

use wavedit::{ WavFile, InstrumentChunk };
use docopt::Docopt;

use std::io;
use std::io::{ Write, Cursor, Read, Error, ErrorKind };
use std::fs::File;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
wavedit

Usage:
  wavedit <sourcedir> <destdir>
  wavedit (-h | --help)
  wavedit --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt
            .version(Some(VERSION.to_string()))
            .parse())
        .unwrap_or_else(|e| e.exit());

    let src = args.get_vec("<sourcedir>");
    let dest = args.get_vec("<destdir>");

    let instrument_name = get_input("instrument name: ");

    let paths = read_directory_paths(Path::new(src[0])).expect("dir to have files");
    for path in paths {
        let mut dest_path = PathBuf::new();
        dest_path.push(dest[0]);

        match path.extension().and_then(|oss| oss.to_str()) {
            Some("wav") => { process_wav(path, instrument_name.as_str(), dest_path) },
            _ => (),
        }
    }

    // let reader = File::open("resources/inst.wav").expect("input wav to read correctly.");
    // let mut wav = WavFile::read(reader).expect("wav to parse correctly");


    // println!("{:?}", input_dir);

    // let paths = read_directory_paths(Path::new(".")).expect("dir to have files");
    // for path in paths {
    //     match path.extension().and_then(|oss| oss.to_str()) {
    //         Some("wav") => { process_wav(path) },
    //         _ => (),
    //     }
    // }

    // let files:Vec<Path> = ::glob::glob("*").collect();
    // println!("{:?}", files);

    // wav.instrument_chunk = Some(InstrumentChunk {
    //     unshifted_note: 45,
    //     fine_tune: 0,
    //     gain: 0,
    //     low_note: 40,
    //     high_note: 50,
    //     low_vel: 10,
    //     high_vel: 120,
    // });

    // use wavedit::{ SamplerChunk, SampleLoop, LoopType };
    // wav.sampler_chunk = Some(SamplerChunk {
    //     manufacturer: 0,
    //     product: 0,
    //     sample_period: 10,
    //     midi_unity_note: 20,
    //     midi_pitch_fraction: 0,
    //     smpte_format: 0,
    //     smpte_offset: 0,
    //     sample_loops: vec![SampleLoop {
    //         id: 0,
    //         loop_type: LoopType::Forward,
    //         start: 10,
    //         end: 1000,
    //         fraction: 0,
    //         play_count: 0,
    //     }],
    //     sampler_data: Vec::new(),
    // });

    // // wav.sampler_chunk = None;
    // // wav.cue_points = Vec::new();

    // // let mut s=String::new();
    // // print!("Low note [0]: ");
    // // let _= io::stdout().flush();
    // // io::stdin().read_line(&mut s).expect("Did not enter a correct string");

    // let writer = File::create("output.wav").expect("output wav to create correctly.");
    // let _ = WavFile::write(writer, wav);
}

pub fn process_wav(path:PathBuf, name:&str, mut dest:PathBuf) {
    println!("\n{}", path.file_name().unwrap().to_str().unwrap());

    let reader = File::open(path).expect("input wav to read correctly.");
    let mut wav = WavFile::read(reader).expect("wav to parse correctly");

    println!("");

    let midi_note = get_input("midi unity note (0-255): ");
    println!("d: {:?}", midi_note.trim());
    let midi_note_number:u8 = midi_note.trim().parse().unwrap();

    println!("using: {:?} as midi unity note.", midi_note_number);

    // use wavedit::{ SamplerChunk, SampleLoop, LoopType };
    // wav.sampler_chunk = Some(SamplerChunk {
    //     manufacturer: 0,
    //     product: 0,
    //     sample_period: 10,
    //     midi_unity_note: midi_note_number as u32,
    //     midi_pitch_fraction: 0,
    //     smpte_format: 0,
    //     smpte_offset: 0,
    //     sample_loops: vec![SampleLoop {
    //         id: 0,
    //         loop_type: LoopType::Forward,
    //         start: 10,
    //         end: 1000,
    //         fraction: 0,
    //         play_count: 0,
    //     }],
    //     sampler_data: Vec::new(),
    // });

    wav.instrument_chunk = Some(InstrumentChunk {
        unshifted_note: 45,
        fine_tune: 0,
        gain: 0,
        low_note: 40,
        high_note: 80,
        low_vel: 0,
        high_vel: 255,
    });

    let note_name = wavedit::note_num_to_name(midi_note_number as u32);
    // let output_file = format!("{}_{}.wav", name.trim(), note_name);
    dest.push(format!("{} {}.wav", name.trim(), note_name));

    println!("writing wav: {:?}", dest);

    let writer = File::create(dest).expect("output wav to create correctly.");
    let _ = WavFile::write(writer, wav);

}

pub fn get_input(prompt:&str) -> String {
    let mut s = String::new();
    print!("{}", prompt);
    let _= io::stdout().flush();
    io::stdin().read_line(&mut s).expect("Did not enter a correct string");
    s
}

use std::path::{ Path, PathBuf };
pub fn read_directory_paths(path:&Path) -> io::Result<Vec<PathBuf>> {
    let mut paths : Vec<PathBuf> = Vec::new();

    for entry in ::std::fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path().to_path_buf();
        paths.push(file_path);
    }

    Ok(paths)
}
