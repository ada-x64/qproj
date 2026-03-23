#!/usr/bin/env bash
set -euo pipefail
if [ $# -eq 0 ]; then
    RUSTFLAGS=-Zcodegen-backend=llvm cargo llvm-cov nextest \
        --manifest-path=./workspace/Cargo.toml \
        --html --open
else
    RUSTFLAGS=-Zcodegen-backend=llvm cargo llvm-cov nextest \
        --manifest-path=./workspace/Cargo.toml \
        "$@"
fi
