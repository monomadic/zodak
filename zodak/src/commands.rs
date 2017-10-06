use wavtag::{ RiffFile, ChunkType, InstrumentChunk, SamplerChunk, SampleLoop, LoopType };
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
                    println!("\nFile: {}", wav.filename);

                    // get the unity note first as it's used in both chunks
                    let unity_note = if args.get_bool("--inst") || args.get_bool("--smpl") {
                        name_to_note_num(&get_input("midi unity note (C0-G8): "))
                    } else { 0 };

                    if args.get_bool("--inst") {
                        let mut inst = wav.get_instrument_chunk();

                        inst.unshifted_note = unity_note;
                        inst.low_note = name_to_note_num(&get_input("midi low note (C0-G8): "));
                        inst.high_note = name_to_note_num(&get_input("midi high note (C0-G8): "));

                        if args.get_bool("--vel") {
                            inst.low_vel = str_to_int(&get_input("midi low vel (0-127): ")) as u8;
                            inst.high_vel = str_to_int(&get_input("midi high vel (0-127): ")) as u8;
                        }

                        wav.set_instrument_chunk(inst);
                    }

                    if args.get_bool("--smpl") {
                        let mut smpl = wav.get_sampler_chunk();

                        smpl.midi_unity_note = unity_note as u32;

                        smpl.sample_loops = vec![SampleLoop {
                            id: 0,
                            loop_type: LoopType::Forward,
                            start: str_to_int(&get_input("loop start (0-4294967294): ")),
                            end: str_to_int(&get_input("loop end (0-4294967294): ")),
                            fraction: 0,
                            play_count: 0,
                        }];

                        wav.set_sampler_chunk(smpl);
                    }

                    let dest = format!("{}", file_name(&wav, instrument_name.as_str()));
                    println!("writing: {}", dest);

                    let mut dest_path = PathBuf::new();
                    dest_path.push(destdir);
                    dest_path.push(dest);

                    // TODO if not read only
                    let writer = fs::File::create(dest_path).expect("output wav to create correctly.");
                    let _ = wav.write(writer);

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
    println!("{}, chunks: {:?}", wav.filename, wav.chunks.len());

    for chunk in wav.chunks {

        match chunk.header {
            ChunkType::Instrument => {
                if let Ok(inst) = InstrumentChunk::from_chunk(&chunk) {
                    println!("{:?}", inst);
                } else { println!("broken inst chunk detected."); }
            },
            ChunkType::Sampler => {
                if let Ok(smpl) = SamplerChunk::from_chunk(&chunk) {
                    println!("{:?}", smpl);
                } else { println!("broken smpl chunk detected."); }
            },
            _ => println!("[{:?}]", chunk.header),
        }

    }
    print!("\n");
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
