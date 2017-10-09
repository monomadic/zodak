use wavtag::{ RiffFile, ChunkType, InstrumentChunk, SamplerChunk, SampleLoop, LoopType };
use wavtag::utils::*;
// use wavtag::{ name_to_note_num, note_num_to_name };
use docopt;
use docopt::Docopt;

use std::io;
use std::io::{ Write };
use std::fs;
use std::path::{ PathBuf, Path };

// #[derive(Debug)]
pub struct DestinationSample {
    pub output_filename: String,
    pub file: RiffFile,
    pub unity_note: u8,
    pub lokey: u8,
    pub hikey: u8,
    pub lovel: u8,
    pub hivel: u8,
    pub loop_start: u32,
    pub loop_end: u32,
}

pub fn run() -> io::Result<()> {
    let args = Docopt::new(::USAGE)
        .and_then(|dopt| dopt
            .version(Some(::VERSION.to_string()))
            .parse())
        .unwrap_or_else(|e| e.exit());

    println!("🎹  ZODAK v{}", ::VERSION);

    // let arg_read_only = args.get_str("--read_only");

    if args.get_bool("print") {
        let sourcedir = args.get_vec("<source>")[0];

        match read_directory(Path::new(sourcedir).to_path_buf()) {
            Ok(wavs) => {
                for wav in wavs { print_wav(wav) }
            },
            Err(_) => println!("No file or directory.") // todo: properly unwrap error message.
        }        
    }

    if args.get_bool("tag") {
        let source_arg = args.get_vec("<source>")[0];
        let destdir = args.get_vec("<destdir>")[0];
        let mut dest_path = PathBuf::new();
        dest_path.push(destdir);

        use std::process::exit;
        if !dest_path.is_dir() {
            println!("\nError: Supplied output path is not a directory.");
            exit(1);
        }

        let mut source = PathBuf::new();
        source.push(source_arg);
        let file_result = if source.is_dir() {
            read_directory(source.clone())
        } else {
            let file = fs::File::open(source.clone()).expect("file to open");
            let filename: String = (*source.file_name().expect("filename to be acceptable").to_string_lossy()).to_string();
            Ok(vec!(RiffFile::read(file, filename).expect("wav to parse correctly")))
        };

        match file_result {
            Ok(wavs) => {
                println!("Found {} wav files.", wavs.len());

                // prompt for an instrument name.
                let instrument_name_default = dir_as_string(source.into_os_string().to_string_lossy().into_owned().as_str()); // todo: clip last directory name
                let mut instrument_name = get_input(format!("instrument name [{}]: ", instrument_name_default).as_str());
                if instrument_name == "" { instrument_name = instrument_name_default };

                fn key_from_filename(filename: &str) -> u8 {
                    use regex::Regex;

                    let re = Regex::new(r"[A-Ga-g][#bB]?\-?[0-8]").expect("regular expression to parse");
                    let capture = &re.captures(filename).expect("regex to find some notes")[0];
                    // println!("Extracted note {:?} from filename {:?}", capture, filename);
                    name_to_note_num(capture)
                }

                fn output_filename(instrument_name: String, keyname: String) -> String {
                    format!("{} {}.wav", instrument_name, keyname)
                }

                fn guess_defaults(wavs: Vec<RiffFile>, instrument_name: String, args: &docopt::ArgvMap) -> Vec<(DestinationSample)> {
                    let mut files_to_write = Vec::new();

                    for wav in wavs {
                        let unity_note_number = key_from_filename(wav.filename.as_str());
                        let unity_note_name = note_num_to_name(unity_note_number as u32);
                        let output_filename = output_filename(instrument_name.clone(), unity_note_name);

                        let loop_start:u32 = if args.get_bool("--loop-start") {
                            str_to_int(args.get_str("--loop-start"))
                        } else {
                            0
                        };

                        let loop_end:u32 = if args.get_bool("--loop-end") {
                            str_to_int(args.get_str("--loop-end"))
                        } else {
                            0
                        };

                        files_to_write.push(DestinationSample {
                            output_filename: output_filename,
                            unity_note: unity_note_number,
                            lokey: 0,
                            hikey: 127,
                            lovel: 0,
                            hivel: 127,
                            file: wav,
                            loop_start: loop_start,
                            loop_end: loop_end,
                        });
                    }

                    files_to_write.sort_by(|a, b| a.unity_note.cmp(&b.unity_note));

                    // we want to sort first THEN ask this info (easier for user).
                    if args.get_bool("--smpl") && (!args.get_bool("--loop-start") || !args.get_bool("--loop-end")) {
                        // found a situation we need to ask for more information.

                        for wav in files_to_write.iter_mut() {
                            println!("\n{}:", wav.file.filename);
                            wav.loop_start = str_to_int(&get_input("loop start (0-4294967294): "));
                            wav.loop_end = str_to_int(&get_input("loop end (0-4294967294): "));
                        }
                    }

                    let num_samples = files_to_write.len();
                    let unity_notes:Vec<u8> = files_to_write.iter().map(|w| w.unity_note ).collect(); // borrow checker shakes fist

                    for (index, file) in files_to_write.iter_mut().enumerate() {
                        if index == 0 {
                            file.lokey = 0
                        } else {
                            file.lokey = file.unity_note
                        }

                        if index+1 == num_samples {
                            // last element
                            file.hikey = 127
                        } else {
                            file.hikey = unity_notes[index+1] - 1
                            // file.hikey = file.next()
                        }
                    }
                    files_to_write
                }

                let defaults = guess_defaults(wavs, instrument_name, &args);

                println!("FILES WRITTEN:");
                print!("{:<40}", "Input");
                print!("{:<40}", "Output");
                print!("{:<15}", "Note");
                print!("{:<15}", "KeyRange");
                print!("{:<15}", "VelRange");
                print!("\n");

                // iterate our guessed defaults, correcting any unwanted info
                for mut wav in defaults {
                    print!("{:<40}", wav.file.filename);
                    print!("{:<40}", wav.output_filename);
                    print!("{:<15}", format!("{} ({})", note_num_to_name(wav.unity_note as u32), wav.unity_note));
                    print!("{:<15}", format!("{}-{}", note_num_to_name(wav.lokey as u32), note_num_to_name(wav.hikey as u32)));
                    print!("{:<15}", format!("{}-{}", wav.lovel, wav.hivel));
                    print!("\n");

                    if args.get_bool("--inst") {
                        wav.file.set_instrument_chunk(
                            InstrumentChunk {
                                unshifted_note: wav.unity_note,
                                fine_tune: 0,
                                gain: 0,
                                low_note: wav.lokey,
                                high_note: wav.hikey,
                                low_vel: wav.lovel,
                                high_vel: wav.hivel,
                            }
                        );
                    }

                    if args.get_bool("--smpl") {
                        wav.file.set_sampler_chunk(
                            SamplerChunk {
                                manufacturer: 0,
                                product: 0,
                                sample_period: 10,
                                midi_unity_note: wav.unity_note as u32,
                                midi_pitch_fraction: 0,
                                smpte_format: 0,
                                smpte_offset: 0,
                                sample_loops: vec![SampleLoop {
                                    id: 0,
                                    loop_type: LoopType::Forward,
                                    start: wav.loop_start,
                                    end: wav.loop_end,
                                    fraction: 0,
                                    play_count: 0,
                                }],
                                sampler_data: Vec::new(),
                            }
                        );
                    }

                    let mut dest_file = dest_path.clone();
                    dest_file.push(wav.output_filename);

                    // TODO if not read only
                    let writer = fs::File::create(dest_file).expect("output wav to create correctly.");
                    let _ = wav.file.write(writer);
                }

                // for mut wav in wavs {
                //     println!("\nFile: {}", wav.filename);

                //     // get the unity note first as it's used in both chunks
                //     let unity_note = if args.get_bool("--inst") || args.get_bool("--smpl") {
                //         name_to_note_num(&get_input("midi unity note (C0-G8): "))
                //     } else { 0 };

                //     // sort out default values from sfz, if provided.
                //     // let _ = if args.get_bool("--sfzinput") {
                //     //     use std::error::Error;

                //     //     let sfzfile = args.get_str("--loop-start");

                //     //     match fs::File::open(&sfzfile) {
                //     //         Err(why) => panic!("Error reading SFZ: {}", why.description()),
                //     //         Ok(file) => InstrumentDefaults::parse_sfz(file).expect("sfz to parse correctly"),
                //     //     }
                //     // } else { InstrumentDefaults::new() };

                //     if args.get_bool("--inst") {
                //         let mut inst = wav.get_instrument_chunk();

                //         inst.unshifted_note = unity_note;
                //         inst.low_note = name_to_note_num(&get_input("midi low note (C0-G8): "));
                //         inst.high_note = name_to_note_num(&get_input("midi high note (C0-G8): "));

                //         if args.get_bool("--vel") {
                //             inst.low_vel = str_to_int(&get_input("midi low vel (0-127): ")) as u8;
                //             inst.high_vel = str_to_int(&get_input("midi high vel (0-127): ")) as u8;
                //         }

                //         wav.set_instrument_chunk(inst);
                //     }

                //     if args.get_bool("--smpl") {
                //         let mut smpl = wav.get_sampler_chunk();

                //         let loop_start:u32 = if args.get_bool("--loop-start") {
                //             str_to_int(args.get_str("--loop-start"))
                //         } else {
                //             str_to_int(&get_input("loop start (0-4294967294): "))
                //         };

                //         let loop_end:u32 = if args.get_bool("--loop-end") {
                //             str_to_int(args.get_str("--loop-end"))
                //         } else {
                //             str_to_int(&get_input("loop end (0-4294967294): "))
                //         };

                //         smpl.midi_unity_note = unity_note as u32;

                //         smpl.sample_loops = vec![SampleLoop {
                //             id: 0,
                //             loop_type: LoopType::Forward,
                //             start: loop_start,
                //             end: loop_end,
                //             fraction: 0,
                //             play_count: 0, // infinite
                //         }];

                //         wav.set_sampler_chunk(smpl);
                //     }

                //     let dest = format!("{}", file_name(&wav, instrument_name.as_str()));
                //     println!("writing: {}", dest);

                //     let mut dest_path = PathBuf::new();
                //     dest_path.push(destdir);
                //     dest_path.push(dest);

                //     // TODO if not read only
                //     let writer = fs::File::create(dest_path).expect("output wav to create correctly.");
                //     let _ = wav.write(writer);

                // }
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
                let filename: String = (*file.file_name().expect("filename to parse correctly").to_string_lossy()).to_string();
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
        current_dir = env::current_dir().expect("current directory to return correctly");
    } else {
        current_dir.push(path);
    }
    current_dir.file_name().expect("current directory to be valid").to_string_lossy().into_owned()
}
