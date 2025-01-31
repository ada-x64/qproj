#!/bin/sh
cargo build --target x86_64-pc-windows-gnu &&
cp target/x86_64-pc-windows-gnu/debug/bevy_game.exe . &&
exec ./bevy_game.exe "$@"
