### dummy #####################################################################

python := ". ./.venv/bin/activate && python3"

set dotenv-load := true
set dotenv-required := false

_check check='' fix='': (_headers fix)
    #!/bin/bash
    . .venv/bin/activate
    set -exuo pipefail
    black {{ check }} ./scripts
    pyright ./scripts
    cargo fmt -- {{ check }}
    cargo clippy --locked --workspace {{ fix }} -- --no-deps
    bevy_lint
    cargo deny check advisories bans sources --hide-inclusion-graph

_headers fix='': _venv
    {{ python }} ./scripts/headers.py {{ fix }}

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
    {{ python }} ./scripts/setup.py {{ ARGS }}

# Run the game with the default flags. On WSL2, this will run ./scripts/wsl.py.
run *ARGS: _venv
    #!/bin/bash
    if [[ "$WSL_DISTRO_NAME" ]]; then
        set -exuo pipefail
        {{ python }} ./scripts/wsl.py {{ ARGS }};
    else
        set -exuo pipefail
        cargo run {{ ARGS }};
    fi

# Runs cargo build with MSVC target env vars set.
build *ARGS:
    cargo build {{ ARGS }}

# Format everything.
fmt: _venv (_headers "--fix")
    cargo fmt
    {{ python }} -m black ./scripts

# Check formatting; lint python, rust, and bevy.
check: (_check "--check" "")

# Runs `just check` with various `--fix` flags enabled.
fix: (_check "" "--fix")

# Test CI locally with nektos/act. Runs ./scripts/ci.py
ci *ARGS: _venv
    {{ python }} ./scripts/ci.py {{ ARGS }}

# Freeze pip requirements.
freeze: _venv
    {{ python }} -m pip freeze --all > requirements.txt

# Runs tests.
test PKG *ARGS:
    cargo nextest run -p {{PKG}} --no-default-features {{ARGS}}

### run ######################################################################

# Run the game with the default flags. On WSL2, this will run ./scripts/wsl.py.
run *ARGS: _venv
    #!/bin/bash
    if [[ "$WSL_DISTRO_NAME" ]]; then
        set -exuo pipefail
        {{ python }} ./scripts/wsl.py {{ ARGS }};
    else
        set -exuo pipefail
        cargo run {{ ARGS }};
    fi

# Plays the game as if in release.
play *ARGS:
    cargo run -F dev --bin qproj {{ARGS}};

# Opens the inspector.
inspect *ARGS:
    cargo run -F dev --bin qproj -F inspector {{ARGS}};

# Runs a tool.
# tool NAME *ARGS:
#     cargo run --bin {{NAME}} {{ARGS}}
