### dummy #####################################################################

python := ". ./.venv/bin/activate && python3"

set dotenv-load := true
set dotenv-required := false

_default:
    just -l

_check check='' fix='': (_headers fix)
    #!/bin/bash
    . .venv/bin/activate
    set -exuo pipefail
    black {{ check }} ./scripts
    pyright ./scripts
    cargo fmt -- {{ check }}
    cargo clippy --locked --workspace {{ fix }} -- --no-deps
    bevy_lint
    cargo deny --workspace -L error check advisories bans sources --hide-inclusion-graph

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

### workflow ##################################################################

# Sets up the development environment. Run with --help for more info.
setup *ARGS: _venv
    {{ python }} ./scripts/setup.py {{ ARGS }}

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

# Runs tests for a specific package
test PKG *ARGS:
    #!/bin/bash
    if [[ "{{PKG}}" = 'all' ]]; then
        cargo nextest run --workspace
    else
        cargo nextest run -p {{PKG}} {{ARGS}}
    fi

# Runs the specified workspace binary.
run BIN *ARGS:
    cargo run {{BIN}} -- {{ARGS}}

# Plays the game.
play *ARGS:
    cargo run quell -- {{ARGS}}

# Runs the inspector
inspect *ARGS:
    cargo run q_inspector -- {{ARGS}}
