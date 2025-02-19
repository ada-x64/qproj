#!/bin/bash
. .env
TEMP="$(wslpath -au "$(pwsh.exe -c 'echo $env:TEMP')")"
HOSTPATH="${HOSTPATH:-$TEMP}" 
TARGET_DIR="${TARGET_DIR:-"target/x86_64-pc-windows-gnu/debug"}"
EXE="${EXE:-"bevy_game.exe"}"
PORT="${PORT:-"1234"}"

cargo build --target x86_64-pc-windows-gnu "$BUILD_FLAGS"
rsync -P "$TARGET_DIR/$EXE" "$HOSTPATH"
ENV_VARS="RUST_LOG=$RUST_LOG,DEBUG_LEVEL=$DEBUG_LEVEL,RUST_BACKTRACE=$RUST_BACKTRACE,DIR=\$env:TEMP,EXE=$EXE"
HOSTNAME="$(ip route show | grep -i default | awk '{ print $3}')"
echo "$ENV_VARS" | nc -t "$HOSTNAME" "$PORT"