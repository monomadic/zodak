#!/bin/sh

cargo build --release --all
cp target/release/zodak ~/.bin/

zodak print _input
