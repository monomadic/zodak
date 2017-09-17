// https://sites.google.com/site/musicgapi/technical-documents/wav-file-format#inst

extern crate wavedit;
// extern crate glob;
extern crate docopt;

use wavedit::{ WavFile, InstrumentChunk };
use wavedit::{ name_to_note_num, note_num_to_name };
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
    // println!("name_to_note_num C-2(0): {:?}\n", name_to_note_num("C-2"));
    // println!("name_to_note_num G#-2(8): {:?}\n", name_to_note_num("G#-2"));
    // println!("name_to_note_num C0(24): {:?}\n", name_to_note_num("C0"));
    // println!("name_to_note_num A2(57): {:?}\n", name_to_note_num("A2"));
    // println!("name_to_note_num A#5(94): {:?}\n", name_to_note_num("A#5"));
    // println!("note_num_to_name C-2(0): {:?}\n", note_num_to_name(0));
    // println!("note_num_to_name G#-2(8): {:?}\n", note_num_to_name(8));
    // println!("note_num_to_name C0(24): {:?}\n", note_num_to_name(24));
    // println!("note_num_to_name A2(57): {:?}\n", note_num_to_name(57));
    // println!("note_num_to_name A#5(94): {:?}\n", note_num_to_name(94));

    println!("WAVEDIT v{}", VERSION);
    println!("using instrument mode.\n");

    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt
            .version(Some(VERSION.to_string()))
            .parse())
        .unwrap_or_else(|e| e.exit());

    let src = args.get_vec("<sourcedir>");
    let dest = args.get_vec("<destdir>");

    let instrument_name_default = dir_as_string(src[0]);
    let mut instrument_name = get_input(format!("instrument name [{}]: ", instrument_name_default).as_str());

    if instrument_name == "" { instrument_name = instrument_name_default };

    let paths = read_directory_paths(Path::new(src[0])).expect("dir to have files");
    for path in paths {
        let mut dest_path = PathBuf::new();
        dest_path.push(dest[0]);

        match path.extension().and_then(|oss| oss.to_str()) {
            Some("wav") => { process_wav(path, instrument_name.as_str(), dest_path) },
            _ => (),
        }
    }
}

pub fn process_wav(path:PathBuf, name:&str, mut dest:PathBuf) {
    println!("\n{}", path.file_name().unwrap().to_str().unwrap());

    let reader = File::open(path).expect("input wav to read correctly.");
    let mut wav = WavFile::read(reader).expect("wav to parse correctly");

    let midi_note_number = get_input("midi unity note (C0-G8): ");
    let midi_note_number = name_to_note_num(&midi_note_number);
    // println!("using midi note number: {:?}", midi_note_number);

    // let midi_note = get_input("midi unity note (C0-G8): ");
    // let midi_note_number:u8 = midi_note.parse().unwrap();

    let midi_low_note_number = get_input("midi low note (C0-G8): ");
    let midi_low_note_number = name_to_note_num(&midi_low_note_number);
    // println!("using midi note number: {:?}", midi_low_note_number);

    // let midi_low_note = get_input("midi low note (0-255): ");
    // let midi_low_note_number:u8 = midi_low_note.parse().unwrap();

    let midi_high_note_number = get_input("midi high note (C0-G8): ");
    let midi_high_note_number = name_to_note_num(&midi_high_note_number);
    // println!("using midi note number: {:?}", midi_high_note_number);

    // let midi_high_note = get_input("midi high note (0-255): ");
    // let midi_high_note_number:u8 = midi_high_note.parse().unwrap();

    let loop_start_str = get_input("loop start (0-4294967294): ");
    let loop_start:u32 = loop_start_str.trim().parse().unwrap();

    let loop_end_str = get_input("loop end (0-4294967294): ");
    let loop_end:u32 = loop_end_str.trim().parse().unwrap();

    use wavedit::{ SamplerChunk, SampleLoop, LoopType };
    wav.sampler_chunk = Some(SamplerChunk {
        manufacturer: 0,
        product: 0,
        sample_period: 10,
        midi_unity_note: midi_note_number as u32,
        midi_pitch_fraction: 0,
        smpte_format: 0,
        smpte_offset: 0,
        sample_loops: vec![SampleLoop {
            id: 0,
            loop_type: LoopType::Forward,
            start: loop_start,
            end: loop_end,
            fraction: 0,
            play_count: 0,
        }],
        sampler_data: Vec::new(),
    });

    wav.instrument_chunk = Some(InstrumentChunk {
        unshifted_note: midi_note_number,
        fine_tune: 0,
        gain: 0,
        low_note: midi_low_note_number,
        high_note: midi_high_note_number,
        low_vel: 0,
        high_vel: 255,
    });

    let note_name = wavedit::note_num_to_name(midi_note_number as u32);
    dest.push(format!("{} {}.wav", name.trim(), note_name));

    // println!("writing wav: {:?}", dest);

    let writer = File::create(dest).expect("output wav to create correctly.");
    let _ = WavFile::write(writer, wav);
}

pub fn get_input(prompt:&str) -> String {
    let mut s = String::new();
    print!("{}", prompt);
    let _= io::stdout().flush();
    io::stdin().read_line(&mut s).expect("Did not enter a correct string");
    s.trim().to_string()
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

pub fn dir_as_string(path:&str) -> String {
    let mut current_dir = PathBuf::new();
    if path == "." {
        use std::env;
        current_dir = env::current_dir().unwrap();
    } else {
        current_dir.push(path);
    }
    current_dir.file_name().expect("current directory to be valid").to_string_lossy().into_owned()
}
