#!/usr/bin/env bash
set -euo pipefail
RUSTC_WRAPPER="" bevy_lint \
    --manifest-path=./workspace/Cargo.toml \
    --all-features \
    --target-dir=target/bevy_lint "$@"
