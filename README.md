# ZODAC - the sample file tool

## Overview

A tool for managing samples, with functionality not found anywhere else, especially not open source. At the moment, the repository supports WAV as an input source and WAV and SFZ as output sources. I was frustrated at complete lack of support for certain aspects of WAV files that makes them extremely useful in various daws.

- meta tagging for cue, loop points, root note, note range, that work in many daws (ableton works 100%, and almost all sampler utilities in every daw or plugin support loop points). This means you can just drag and drop your wavs into ableton and they're automatically mapped, freeing up the format and your sample lib a bit from internal garbage formats (hello kontakt).
- sfz export

At the moment, the front end command line interface is simple to use, but the underlying tool has a lot of functionality not exposed by it (just wanted to keep the front end simple for now).

## Eventual support:

- sf2/sf3/sf4 instruments
- flac/ogg formats
- I really need help on this but I'd like to openly support non-encrypted kontakt files, exs, etc. Lets free up these sample formats and in turn free up the audio industry, by documenting their structures in code here. This does NOT violate any laws despite what forum posters will have you think, you are transcribing/converting your own files for your own use. Don't have them tied up in garbage formats that leave your projects broken with missing samples and version mismatches when you come back to them.

## Other goals:
- a 'library' mode, which audits your entire library for problems and keeps all of your instruments organised.
- single shot mode
- drum kit mode

## Usage:
```shell
WAVEDIT v0.1.0
using instrument mode.

Invalid arguments.

Usage:
  wavedit <sourcedir> <destdir>
  wavedit (-h | --help)
  wavedit --version
  ```