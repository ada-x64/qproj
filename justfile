### dummy #####################################################################

python := ". ./.venv/bin/activate && python3"

set dotenv-load := true
set dotenv-required := false

_default:
    just -l

_venv:
    #!/bin/bash
    set -euo pipefail
    if [ ! -d .venv ]; then
        echo "Setting up virtual environment..."
        python3 -m venv .venv
        . .venv/bin/activate
        python3 -m pip install -r requirements.txt
    fi

_chk_deps:
    cargo deny --workspace -L error check advisories bans sources --hide-inclusion-graph

_chk_python check='':
    #!/bin/bash
    . .venv/bin/activate
    set -exuo pipefail
    black {{ check }} ./scripts
    pyright ./scripts

_chk_rust check='' fix='':
    cargo fmt -- {{ check }}
    cargo clippy --locked --workspace {{ fix }} -- --no-deps
    bevy_lint

_chk_headers fix='': _venv
    {{ python }} ./scripts/headers.py {{ fix }}


### workflow ##################################################################

# Sets up the development environment. Run with --help for more info.
[group('workflow')]
setup *ARGS: _venv
    {{ python }} ./scripts/setup.py {{ ARGS }}

# Format everything.
[group('workflow')]
fmt: _venv (_chk_headers "--fix")
    cargo fmt
    {{ python }} -m black ./scripts

# Check formatting; lint python, rust, and bevy.
[group('workflow')]
check: (_chk_rust "--check") (_chk_python "--check") (_chk_headers) (_chk_deps)

# Runs `just check` with various `--fix` flags enabled.
[group('workflow')]
fix: (_chk_rust "" "--fix") (_chk_python) (_chk_headers "--fix")

# Test CI locally with nektos/act. Runs ./scripts/ci.py
[group('workflow')]
ci *ARGS: _venv
    {{ python }} ./scripts/ci.py {{ ARGS }}

# Freeze pip requirements.
[group('workflow')]
freeze: _venv
    {{ python }} -m pip freeze --all > requirements.txt

# Runs tests for a specific package
[group('workflow')]
test PKG *ARGS:
    #!/bin/bash
    if [[ "{{PKG}}" = 'all' ]]; then
        cargo nextest run --workspace
    else
        cargo nextest run -p {{PKG}} {{ARGS}}
    fi


### run #######################################################################

# Runs the specified workspace binary.
[group('runners')]
run BIN *ARGS:
    cargo run -F dev --bin {{BIN}} -- {{ARGS}}

# Plays the game.
[group('runners')]
play *ARGS: (run "q_app")

# Runs the inspector
[group('runners')]
inspect *ARGS: (run "q_inspector")
