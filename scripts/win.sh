#!/bin/sh
cargo build --target x86_64-pc-windows-gnu 
# cp target/x86_64-pc-windows-gnu/debug/bevy_game.exe .
# cp target/x86_64-pc-windows-gnu/debug/bevy_dylib.dll .
cd target/x86_64-pc-windows-gnu/debug || exit
exec ./bevy_game.exe "$@"
