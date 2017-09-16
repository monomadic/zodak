#!/bin/sh

cargo build --release --all
cp target/release/wavedit ~/.bin/

wavedit /Users/rob/Music/Kontakt/Retro\ Machines\ Mk2\ Library/Analog\ Machines/Catchy\ Filter\ Bass\ Samples _output
