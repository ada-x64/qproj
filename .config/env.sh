export CARGO_TERM_COLOR="always"
export PYTHONUNBUFFERED=1
export RUSTC_WRAPPER="sccache"
export MISE_EXPERIMENTAL=1

if [[ -n "$SSH_CLIENT" ]]; then
   export FEATURES=""
else
    export FEATURES="dylib"
fi

if [[ -n "$CI" ]]; then
    export MISE_ENV="ci"
elif [[ -n "$SSH_CLIENT" ]]; then
    export MISE_ENV="ssh"
fi
