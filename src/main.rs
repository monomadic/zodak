// https://sites.google.com/site/musicgapi/technical-documents/wav-file-format#inst

extern crate zodak;
// extern crate glob;
extern crate docopt;

use zodak::{ WavFile, InstrumentChunk };
use zodak::{ name_to_note_num, note_num_to_name };
use docopt::Docopt;

use std::io;
use std::io::{ Write, Cursor, Read, Error, ErrorKind };
use std::fs::File;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
ZODAK 🐉🎹

Usage:
  zodak tag <sourcedir> <destdir> [--start=<n>] [--end=<n>]
  zodak print <sourcedir>
  zodak (-h | --help)
  zodak --version

Options:
  -h --help             Show this screen.
  --version             Show version.
  --start=<n>           Override loop start for all files processed
  --end=<n>             Override loop end for all files processed
  --overwrite           Prompt to overwrite tags already within the WAV source (default=off)
  --velocity            Prompt for a velocity range for each sample
  --sfz                 Output an SFZ file with data from the input files
  --sfzinput=<file>     Use an SFZ as an override for all tags
  --readonly
";

fn read_directory(path:PathBuf) -> Vec<WavFile> {
    let mut wavs = Vec::new();

    for file in read_directory_paths(&path).expect("dir to have files") {
        match file.extension().and_then(|oss| oss.to_str()) {
            Some("wav") => {
                let reader = File::open(file).expect("input wav to read correctly.");
                wavs.push(WavFile::read(reader).expect("wav to parse correctly"));
            },
            _ => ()
        }
    }

    wavs
}

// use std::fmt;
// impl fmt::Display for WavFile {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "WAVE")
//     }
// }

fn print_wav(wav:WavFile) {
    println!("=========");
    println!("wav!");
    println!("- FMT: (len={})", wav.format_chunk.len());
    println!("- DATA: (len={})", wav.data_chunk.len());
    println!("- SMPL: {:?})", wav.sampler_chunk);
    println!("=========");
}

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

    println!("ZODAK v{} 🐉🎹", VERSION);
    println!("using instrument mode.\n");

    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt
            .version(Some(VERSION.to_string()))
            .parse())
        .unwrap_or_else(|e| e.exit());

    let src = args.get_vec("<sourcedir>");
    let dest = args.get_vec("<destdir>");

    let print_option = args.find("print").unwrap().as_bool();

    if print_option {
        println!("printing");

        let wavs = read_directory(Path::new(src[0]).to_path_buf());
        for wav in wavs { print_wav(wav) }
    }

    // let read_only = args.find("--readonly").unwrap().as_bool();

    let arg_loop_start = args.get_str("--start");
    let arg_loop_end = args.get_str("--end");

    let instrument_name_default = dir_as_string(src[0]);
    let mut instrument_name = get_input(format!("instrument name [{}]: ", instrument_name_default).as_str());

    if instrument_name == "" { instrument_name = instrument_name_default };

    // SFZ
    let mut sfzfile = PathBuf::new();
    sfzfile.push(dest[0]);
    sfzfile.push(instrument_name.as_str());
    sfzfile.set_extension("sfz");
    let mut sfz = File::create(sfzfile).expect("sfz to create for writing");

    let paths = read_directory_paths(Path::new(src[0])).expect("dir to have files");
    for path in paths {
        let mut dest_path = PathBuf::new();
        dest_path.push(dest[0]);

        match path.extension().and_then(|oss| oss.to_str()) {
            Some("wav") => {
                println!("\n{}", path.file_name().unwrap().to_string_lossy());
                let reader = File::open(path).expect("input wav to read correctly.");
                let mut wav = WavFile::read(reader).expect("wav to parse correctly");

                process_wav(&mut wav, instrument_name.as_str(), &mut dest_path, arg_loop_start.to_string(), arg_loop_end.to_string());
                append_sample_to_sfz(&mut sfz, &wav, &dest_path);
            },
            _ => (),
        }
    }

}

pub fn append_sample_to_sfz(sfz:&mut File, wav:&WavFile, mut dest:&PathBuf) {
    writeln!(sfz, "<region>");
    writeln!(sfz, "sample={}", dest.file_name().unwrap().to_string_lossy().into_owned());
    writeln!(sfz, "pitch_keycenter={}", wav.pitch_keycenter().expect("wav object to contain a key center"));

    let key_range = wav.key_range().expect("wav object to contain a key range");
    writeln!(sfz, "lokey={} hikey={}", key_range.0, key_range.1);

    let loop_points = wav.loop_points().expect("wav object to contain loop points");
    writeln!(sfz, "loop_mode=loop_continuous loop_start={} loop_end={}\n", loop_points.0, loop_points.1);
}

pub fn process_wav(wav:&mut WavFile, name:&str, mut dest:&mut PathBuf, mut arg_loop_start:String, mut arg_loop_end:String) {
    // let reader = File::open(path).expect("input wav to read correctly.");
    // let mut wav = WavFile::read(reader).expect("wav to parse correctly");

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

    if arg_loop_start == "" {
        arg_loop_start = get_input("loop start (0-4294967294): ");
    };
    let loop_start:u32 = arg_loop_start.trim().parse().unwrap();

    if arg_loop_end == "" {
        arg_loop_end = get_input("loop end (0-4294967294): ");
    };
    let loop_end:u32 = arg_loop_end.trim().parse().unwrap();

    use zodak::{ SamplerChunk, SampleLoop, LoopType };
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

    // let note_name = zodak::note_num_to_name(midi_note_number as u32);
    // dest.push(format!("{} {}.wav", name.trim(), note_name));

    dest.push(wav.file_name(name));

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