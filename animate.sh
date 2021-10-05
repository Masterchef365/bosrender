#!/usr/bin/bash
cargo run --release -- $1 -f $2 --name out --tile-width 500 --tile-height 500 &&
ffmpeg -i out_%4d.png -framerate 60 $(basename $1).mp4 #&&
#rm out_*.png
