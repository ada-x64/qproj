### setup #####################################################################

wsl := env("WSL_DISTRO_NAME", "")

### dummy #####################################################################

_check check='' fix='': (_headers fix)
    #!/bin/bash
    . .venv/bin/activate
    set -exuo pipefail
    black {{ check }} ./scripts
    pyright ./scripts
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

_msvc:
    #!/bin/bash
    . ./.env.msvc

### user ######################################################################

# Sets up the development environment. Run with --help for more info.
setup *ARGS: _venv
    .venv/bin/python ./scripts/setup.py {{ ARGS }}

# Run the game with the default flags. On WSL2, this will run ./scripts/wsl.py.
run *ARGS: _venv _msvc
    if [[ "{{ wsl }}" ]]; then .venv/bin/python ./scripts/wsl.py {{ ARGS }}; else cargo run {{ ARGS }}; fi

# Just runs cargo build.
build *ARGS: _msvc
    cargo build {{ ARGS }}

# Format everything.
fmt: _venv (_headers "--fix")
    cargo fmt
    black ./scripts

# Check formatting; lint python, rust, and bevy.
check: (_check "--check" "")

# Runs `just check` with various `--fix` flags enabled.
fix: (_check "" "--fix")

# Test CI locally with nektos/act. Runs ./scripts/ci.py
ci *ARGS: _venv
    .venv/bin/python ./scripts/ci.py {{ ARGS }}

# Freeze pip requirements.
freeze: _venv
    pip freeze --all > requirements.txt
