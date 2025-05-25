### dummy #####################################################################

wsl := env("WSL_DISTRO_NAME", "")

_check check fix: _headers
    cargo clippy --locked --workspace {{ fix }} -- --no-deps
    bevy_lint
    cargo fmt -- {{ check }}

_headers:
    ./scripts/headers.sh

### user ######################################################################

setup:
    ./scripts/setup.sh

run *ARGS:
    if [[ "{{ wsl }}" ]]; then .venv/bin/python3 ./scripts/wsl.py {{ ARGS }}; else cargo run {{ ARGS }}; fi

check: (_check "true" "false")

fix: (_check "false" "true")

ci:
    .venv/bin/python3 ./scripts/ci.py
