#!/usr/bin/env bash
set -euo pipefail
cargo build --manifest-path=./workspace/Cargo.toml "$@"
