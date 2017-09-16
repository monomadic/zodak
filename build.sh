#!/bin/sh

cargo build --release --all
cp target/release/wavedit ~/.bin/

wavedit _input _output
