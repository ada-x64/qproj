#!/usr/bin/env bash
set -euo pipefail
cargo deny --workspace --manifest-path=./workspace/Cargo.toml \
    -L error \
    check advisories bans sources \
    --hide-inclusion-graph
