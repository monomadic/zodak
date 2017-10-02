extern crate wavtag;
extern crate docopt;

mod commands;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const USAGE: &'static str = "
🎹  ZODAK

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

fn main() {
    ::commands::run().expect("success");
}
