#!/bin/bash

TARGET_DIR="target/x86_64-pc-windows-gnu/debug"
EXE="bevy_game.exe"

. .env
cargo build --target x86_64-pc-windows-gnu "$BUILD_FLAGS"
echo "waiting for launch..."
exec "$TARGET_DIR/$EXE"