extern crate wavtag;
extern crate docopt;

mod commands;
mod sfz;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const USAGE: &'static str = "
🎹  ZODAK

Usage:
  zodak tag <sourcedir> <destdir> [--inst] [--smpl] [--sfz] [--override-loop-start=<n>] [--override-loop-end=<n>]
  zodak print <sourcedir>
  zodak (-h | --help)
  zodak --version

Options:
  -h --help             Show this screen.
  --version             Show version.

  --overwrite           Prompt to overwrite tags already within the WAV source (default=off)
  --velocity            Prompt for a velocity range for each sample (default=off)
  --readonly

  --inst                Add or edit instrument chunk
  --smpl                Add or edit sampler chunk

  --sfz                 Output an SFZ file with data from the input files
  --sfzinput=<file>     Use an SFZ as an override for all tags

  --override-loop-start=<n>           Override loop start for all files processed
  --override-loop-end=<n>             Override loop end for all files processed
  
";

fn main() {
    ::commands::run().expect("success");
}
