### dummy #####################################################################

wsl := env("WSL_DISTRO_NAME", "")

_check check='' fix='': (_headers fix)
    .venv/bin/python3 -m black {{ check }} ./scripts
    .venv/bin/python3 -m pyright ./scripts
    cargo fmt -- {{ check }}
    cargo clippy --locked --workspace {{ fix }} -- --no-deps
    bevy_lint

_headers fix='': _venv
    .venv/bin/python3 ./scripts/headers.py {{ fix }}

_venv:
    #!/bin/bash
    set -euo pipefail
    if [ ! -d .venv ]; then
        echo "Setting up virtual environment..."
        python3 -m venv .venv
        . .venv/bin/activate
        python3 -m pip install -r requirements.txt
    fi

### user ######################################################################

# Sets up the development environment. Run with --help for more info.
setup *ARGS: _venv
    .venv/bin/python ./scripts/setup.py {{ ARGS }}

run *ARGS: _venv
    if [[ "{{ wsl }}" ]]; then .venv/bin/python ./scripts/wsl.py {{ ARGS }}; else cargo run {{ ARGS }}; fi

fmt: _venv (_headers "--fix")
    cargo fmt
    black ./scripts

check: (_check "--check" "")

fix: (_check "" "--fix")

ci: _venv
    .venv/bin/python ./scripts/ci.py
