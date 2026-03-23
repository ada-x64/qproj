#!/usr/bin/env bash
set -ex
if [ $# -eq 0 ]; then
    cargo nextest \
        --config-file=./.config/nextest.toml \
        --manifest-path=./workspace/Cargo.toml \
        r --workspace
else
    cargo nextest \
        --config-file=./.config/nextest.toml \
        --manifest-path=./workspace/Cargo.toml \
        "$@"
fi
