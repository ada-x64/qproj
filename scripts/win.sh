#!/bin/sh
export RUST_BACKTRACE=1
cargo build --target x86_64-pc-windows-gnu &&
cd target/x86_64-pc-windows-gnu/debug &&
exec ./bevy_game.exe "$@"
