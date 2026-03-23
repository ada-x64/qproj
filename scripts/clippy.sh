#!/usr/bin/env bash
set -euo pipefail
cargo clippy --manifest-path=./workspace/Cargo.toml --all-features \
    --target-dir=target/clippy "$@"
