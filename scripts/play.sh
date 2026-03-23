#!/usr/bin/env bash
set -e

EXAMPLE=""
BUILD_ARGS=""
FILE="workspace/target/debug/app"
ENV_VARS=""
CMD_ARGS=""
ASSETS="./workspace/assets"

while [[ $# -gt 0 ]]; do
    case "$1" in
        -x|--example) EXAMPLE="$2"; shift 2 ;;
        -B|--build-args) BUILD_ARGS="$2"; shift 2 ;;
        -e|--env) ENV_VARS="$2"; shift 2 ;;
        -a|--args) CMD_ARGS="$2"; shift 2 ;;
        -A|--assets) ASSETS="$2"; shift 2 ;;
        *) FILE="$1"; shift ;;
    esac
done

if [ -n "$EXAMPLE" ]; then
    CARGO_EXAMPLE="--example $EXAMPLE"
fi

just build ${BUILD_ARGS:-} ${CARGO_EXAMPLE:-}

if [ -n "$EXAMPLE" ]; then
    TARGET_PATH="./workspace/target/debug/examples/$EXAMPLE"
else
    TARGET_PATH="$FILE"
fi

if [ -n "${SSH_CLIENT:-}" ]; then
    set -x
    PSYNC_LOG=debug uvx --from cubething_psync psync \
        "$TARGET_PATH" -e "${ENV_VARS}" -a "${CMD_ARGS}" -A "${ASSETS}"
else
    set -x
    exec "$TARGET_PATH" ${CMD_ARGS}
fi
