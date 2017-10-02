use wavtag::{ RiffFile };
// use wavtag::{ name_to_note_num, note_num_to_name };
use docopt::Docopt;

use std::io;
use std::fs;
use std::path::{ PathBuf, Path };

pub fn run() -> io::Result<()> {
    let args = Docopt::new(::USAGE)
        .and_then(|dopt| dopt
            .version(Some(::VERSION.to_string()))
            .parse())
        .unwrap_or_else(|e| e.exit());

    println!("🎹  ZODAK v{}", ::VERSION);

    let arg_read_only = args.get_str("--read_only");

    if args.get_bool("print") {
        let sourcedir = args.get_vec("<sourcedir>");

        match read_directory(Path::new(sourcedir[0]).to_path_buf()) {
            Ok(wavs) => {
                for wav in wavs { print_wav(wav) }
            },
            Err(_) => println!("No file or directory.") // todo: properly unwrap error message.
        }

        // let wavs = read_directory(Path::new(sourcedir[0]).to_path_buf())?;
        
    }

    Ok(())
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
