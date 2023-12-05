mod commands;
mod midi;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
🎹  ZODAK

Usage:
  zodak tag <source> <destdir> [--inst] [--smpl] [--sfz] [--guess-keymap] [--sfzinput=<file>] [--loop-start=<n>] [--loop-end=<n>] [--verbose]
  zodak print <source>
  zodak (-h | --help)
  zodak --version

Options:
  -h --help             Show this screen.
  --version             Show version.

  --overwrite           Prompt to overwrite tags already within the WAV source (default=off)
  --velocity            Prompt for a velocity range for each sample (default=off)
  --readonly
  --verbose             Display more information during parsing

  --guess-keymap        Attempt to guess a keymap based on filenames

  --inst                Add or edit instrument chunk
  --smpl                Add or edit sampler chunk

  --sfz                 Output an SFZ file with data from the input files
  --sfzinput=<file>     Use an SFZ as an override for all tags

  --loop-start=<n>           Override loop start for all files processed
  --loop-end=<n>             Override loop end for all files processed

";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(crate::commands::run()?)
}
