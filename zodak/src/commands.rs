use wavtag::{ RiffFile, ChunkType };
use wavtag::utils::*;
// use wavtag::{ name_to_note_num, note_num_to_name };
use docopt::Docopt;

use std::io;
use std::io::{ Write };
use std::fs;
use std::path::{ PathBuf, Path };

pub fn run() -> io::Result<()> {
    let args = Docopt::new(::USAGE)
        .and_then(|dopt| dopt
            .version(Some(::VERSION.to_string()))
            .parse())
        .unwrap_or_else(|e| e.exit());

    println!("🎹  ZODAK v{}", ::VERSION);

    // let arg_read_only = args.get_str("--read_only");

    if args.get_bool("print") {
        let sourcedir = args.get_vec("<sourcedir>")[0];

        match read_directory(Path::new(sourcedir).to_path_buf()) {
            Ok(wavs) => {
                for wav in wavs { print_wav(wav) }
            },
            Err(_) => println!("No file or directory.") // todo: properly unwrap error message.
        }        
    }

    if args.get_bool("tag") {
        let sourcedir = args.get_vec("<sourcedir>")[0];
        let destdir = args.get_vec("<destdir>")[0];

        match read_directory(Path::new(sourcedir).to_path_buf()) {
            Ok(wavs) => {
                // prompt for an instrument name.
                let instrument_name_default = dir_as_string(sourcedir); // todo: clip last directory name
                let mut instrument_name = get_input(format!("instrument name [{}]: ", instrument_name_default).as_str());
                if instrument_name == "" { instrument_name = instrument_name_default };

                for mut wav in wavs {
                    println!("{}", wav.filename);

                    let arg_inst = args.get_bool("--inst");

                    if arg_inst {
                        println!("instrument chunk");

                        let mut inst = wav.get_instrument_chunk();

                        let midi_note_number = get_input("midi unity note (C0-G8): ");
                        inst.unshifted_note = name_to_note_num(&midi_note_number);

                        wav.set_instrument_chunk(inst);
                    }

                    let dest = format!("{}", file_name(&wav, instrument_name.as_str()));
                    println!("writing: {}", dest);

                    // TODO if not read only
                    let mut writer = fs::File::create(dest).expect("output wav to create correctly.");
                    wav.write(writer);

                }
            },
            Err(_) => println!("No file or directory.") // todo: properly unwrap error message.
        }
    }

    Ok(())
}

pub fn get_input(prompt:&str) -> String {
    let mut s = String::new();
    print!("{}", prompt);
    let _= io::stdout().flush();
    io::stdin().read_line(&mut s).expect("Did not enter a correct string");
    s.trim().to_string()
}

fn read_directory(path:PathBuf) -> io::Result<Vec<RiffFile>>  {
    pub fn path_to_pathbufs(path:&Path) -> io::Result<Vec<PathBuf>> {
        Ok(fs::read_dir(path)?.map(|file| {
            file.expect("path to unwrap").path().to_path_buf()
        }).collect())
    }

    Ok(path_to_pathbufs(&path)?.iter().filter_map(|file| {
        match file.extension().and_then(|oss| oss.to_str()) {
            Some("wav") => {
                let reader = fs::File::open(file).expect("file to open");
                let filename: String = (*file.file_name().unwrap().to_string_lossy()).to_string();
                Some(RiffFile::read(reader, filename).expect("wav to parse correctly"))
            },
            _ => None
        }
    }).collect())
}

fn print_wav(wav:RiffFile) {
    print!("{}, chunks: {:?}", wav.filename, wav.len());
    for chunk in wav.chunks {
        print!(" [{:?}]", chunk.header);
    }
    print!("\n");
    // println!("- FMT: (len={})", wav.format_chunk.len());
    // println!("- DATA: (len={})", wav.data_chunk.len());
    // println!("- SMPL: {:?})", wav.sampler_chunk);
}

pub fn file_name(wav: &RiffFile, name:&str) -> String {
    if wav.find_chunk_by_type(ChunkType::Instrument).is_some() {
        let note_name = note_num_to_name(wav.get_instrument_chunk().unshifted_note as u32); // midi_unity_note?
        format!("{} {}.wav", name.trim(), note_name)
    } else {
        format!("{}.wav", name.trim())
    }
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
