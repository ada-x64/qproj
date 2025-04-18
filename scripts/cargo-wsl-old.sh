#!/bin/bash
if [ "$2" = "-h" ] || [ "$2" = "--help" ]; then
  cat <<EOF
cargo wsl [-h --help] <ARGS>
This script builds and runs the game assuming you are running in a wsl environment.
System dependencies: xwin, rsync.
Recommended dependencies: windbgx, renderdoc.
Be sure to run \`cargo setup\` before running this script!
<ARGS> are passed directly to cargo.

Accepts the following environment variables:
NOBUILD=<boolean>   : Do not build the executable.
NOSYNC=<boolean>    : Do not sync to the host environment.
NORUN=<boolean>     : Do not run the final executable.
HOSTPATH=<path>     : Directory for the final executable in "wslpath -au" style. Defaults to \$env:TEMP.
TARGET_DIR=<path>   : Target directory for the build.
PKG_NAME=<string>   : Name of the package to build.
BIN_NAME=<string>   : Name of the binary to build.
XWIN_DIR=<path>     : Absolute path for the xwin splat. Defaults to ./.xwin-cache/splat.
WINDBG=<boolean>    : Run windbgx.exe to debug the application.
RENDERDOC=<boolean> : Run qrenderdoc.exe to debug the application.
WINDBG_EXE=<path>   : The windbg executable. Defaults to windbgx.exe
RENDERDOC_EXE=<path>: The qrenderdoc executable. Defaults to qrenderdoc.exe
CARGO_CMD=<string>  : The cargo command to execute. Defaults to "build."
                      Setting this variable will automatically set NOSYNC=true NORUN=true

Example:
# Run the 'terrain-editor' bin from the 'tools' package.
PKG_NAME='tools' BIN_NAME='terrain-editor' cargo wsl
# Run windbg without building.
NOBUILD=true NOSYNC=true WINDBG=true cargo wsl
EOF
  exit 0
fi

shopt -s globstar
. .env
PKG_NAME="${PKG_NAME:-"bevy_game"}"
BIN_NAME="${BIN_NAME:-"bevy_game"}"
TARGET_DIR="${TARGET_DIR:-"target/x86_64-pc-windows-msvc/debug"}"
if [ -n "$CARGO_CMD" ]; then
  NORUN="true"
  NOSYNC="true"
fi
CARGO_CMD="${CARGO_CMD:-"build"}"

if [ -n "$SSH_CONNECTION" ]; then
  SSH_PORT=${SSH_PORT:-$(echo "$SSH_CONNECTION" | cut -d ' ' -f 2)}
  SSH_IP=${SSH_IP:-$(echo "$SSH_CONNECTION" | cut -d ' ' -f 1)}
  SSH_USER=${SSH_USER:-$(whoami)}
  SSH_PATH=${SSH_PATH:-"C:/Users/$SSH_USER/$PKG_NAME"}
  SSH_DEST="scp://$SSH_USER@$SSH_IP:$SSH_PORT/$SSH_PATH"
  echo "SSH CLIENT CONNECTED."
  echo "DESTINATION: $SSH_DEST"
else
  TEMP="$(wslpath -au "$(pwsh.exe -c "echo \$env:TEMP")")"
  TEMP="$(echo "$TEMP" | tr -d '\r\n')"
  HOSTPATH="${HOSTPATH:-$TEMP/$PKG_NAME}"
fi

# Config from .env
# includes RUSTFLAGS and LINKER
ARGS="${@:2}"
if [ -z "$NOBUILD" ]; then
  CMD="cargo $CARGO_CMD    -F 'debug'\
    -F 'inspector'\
    -p '$PKG_NAME'\
    --bin '$BIN_NAME'\
    $ARGS"
  echo "$CMD"
  eval "$CMD"

  RES=$?
  if [ $RES -ne 0 ]; then
    echo Build failed. Did you run \`cargo setup\`?
    exit $RES
  fi
fi

if [ -z "$NOSYNC" ]; then
  if [ -n "$SSH_CONNECTION" ]; then

    # zip -r "$TARGET_DIR/$BIN_NAME.zip" "$FILE_LIST $TARGET_DIR/chk.csv"
    # scp -r "$TARGET_DIR/$BIN_NAME.zip" "$SSH_DEST"
    if [ -n "$SSH_EXTRA_COMMAND" ]; then bash -c "$SSH_EXTRA_COMMAND"; fi
  else
    rsync -avP --no-r "$TARGET_DIR"/* "$HOSTPATH"
    rsync -avP "assets"/* "$HOSTPATH/assets"
  fi
fi

if [ -z "$NORUN" ] && [ -z "$SSH_CONNECTION" ]; then
  HOST_EXE="$(wslpath -aw "$HOSTPATH/$BIN_NAME.exe")"
  START="\
  Start-Process -NoNewWindow -Wait -Environment @{
  RUST_LOG='$RUST_LOG'
  DEBUG_LEVEL='$DEBUG_LEVEL'
}"
  TTY="$(wslpath -aw "$(tty)")"
  REDIRECT="*> '$TTY'"
  if [ -n "$WINDBG" ]; then
    HOST_SRCPATH="$(
      find ./crates ./src -name "*.rs" -type f -print0 |
        xargs -0 -I{} dirname {} |
        sort |
        uniq |
        xargs -I{} wslpath -aw {} |
        tr '\n' ';'
    )"
    WINDBG_EXE=${WINDBG_EXE:-"windbgx.exe"}
    # todo: env vars?
    WINDBG_CMD="$START -FilePath '$WINDBG_EXE' -ArgumentList @('-srcpath \"$HOST_SRCPATH\"', '$HOST_EXE') $REDIRECT"
    echo "pwsh.exe -c \"$WINDBG_CMD\""
    pwsh.exe -c "$WINDBG_CMD"
  elif [ -n "$RENDERDOC" ]; then
    RENDERDOC_EXE=${RENDERDOC_EXE:-"qrenderdoc.exe"}
    CMD="$START -FilePath '$RENDERDOC_EXE' -ArgumentList @('$HOST_EXE') $REDIRECT"
    echo "pwsh.exe -c \"$CMD\""
    pwsh.exe -c "$CMD"
  else
    CMD="$START -FilePath '$HOST_EXE' $REDIRECT"
    echo "pwsh.exe -c \"$CMD\""
    pwsh.exe -c "$CMD"
  fi

fi
