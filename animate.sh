#!/usr/bin/bash
cargo run --release -- $1 -f $2 --name out &&
ffmpeg -i out_%3d.png -framerate 60 $(basename $1).mp4 #&&
#rm out_*
