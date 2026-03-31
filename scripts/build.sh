#!/usr/bin/env bash
set -euo pipefail

EXAMPLE=""
DYLIB="-F dylib"
BUILD_ARGS=""
FILE="workspace/target/debug/app"
ENV_VARS=""
CMD_ARGS=""
ASSETS="./workspace/assets"
RELEASE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        -x|--example) EXAMPLE="$2"; shift 2 ;;
        -B|--build-args) BUILD_ARGS="$2"; shift 2 ;;
        -e|--env) ENV_VARS="$2"; shift 2 ;;
        -a|--args) CMD_ARGS="$2"; shift 2 ;;
        -A|--assets) ASSETS="$2"; shift 2 ;;
        -r|--release) RELEASE="--release"; DYLIB=""; shift ;;
        -D|--no-dylib) DYLIB=""; shift ;;
        *) FILE="$1"; shift ;;
    esac
done

if [ -n "$EXAMPLE" ]; then
    EXAMPLE="--example $EXAMPLE"
fi

cargo build --manifest-path=./workspace/Cargo.toml $BUILD_ARGS $EXAMPLE $RELEASE $DYLIB
