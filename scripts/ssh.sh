#!/bin/bash

PKG_NAME="${PKG_NAME:-"bevy_game"}"
BIN_NAME="${BIN_NAME:-"bevy_game"}"
TARGET_DIR="${TARGET_DIR:-"target/x86_64-pc-windows-msvc/debug"}"

SSH_PORT=${SSH_PORT:-$(echo "$SSH_CONNECTION" | cut -d ' ' -f 2)}
SSH_IP=${SSH_IP:-$(echo "$SSH_CONNECTION" | cut -d ' ' -f 1)}
SSH_USER=${SSH_USER:-$(whoami)}
SSH_PATH=${SSH_PATH:-"C:/Users/$SSH_USER/$PKG_NAME"}
SSH_DEST="scp://$SSH_USER@$SSH_IP:$SSH_PORT/$SSH_PATH"

START="\
  Start-Process -Wait -Environment @{
  RUST_LOG='$RUST_LOG'
  DEBUG_LEVEL='$DEBUG_LEVEL'
} -FilePath '$SSH_PATH/$BIN_NAME.exe'"
echo "$START" >"$TARGET_DIR/start.ps1"

FILE_LIST=(
  "$TARGET_DIR/start.ps1"
  "$TARGET_DIR/$BIN_NAME.exe"
  ".env"
  ".env.local"
)
mapfile -O 4 -t FILE_LIST < <(find assets -type f)

# check chksum if exists, only process changed files
OUTDIR="build"
mkdir -p "$OUTDIR/br"

CHKSUM=$(rhash --md5 -r "${FILE_LIST[@]}")
if [ -e "$OUTDIR/chksum" ]; then
  # REGEX="^(.+)\s+ERR$"
  # CHANGED=$(rhash -c --brief --skip-ok $OUTDIR/chksum | sed -nr "s/$REGEX/\1/p")
  DIFF=$(sort <(echo "$CHKSUM") $OUTDIR/chksum | uniq -u)
  echo -e "DIFF:\n$DIFF"
  CHANGED="$(echo "$DIFF" | sed -nr "s/\s+(.+)$/\1/p")"
  echo -e "CHANGED FILES:\n$CHANGED"
  mapfile -t FILE_LIST < <(echo "$CHANGED")
fi
echo "$CHKSUM" >"$OUTDIR/chksum"

if [ "${FILE_LIST[0]}" ]; then

  for F in "${FILE_LIST[@]}"; do
    if [[ "$F" == "$TARGET_DIR"* ]]; then
      OUTFILE=$(basename "$F")
    else
      OUTFILE="$F"
    fi
    if [[ "$F" == "assets"* ]]; then
      mkdir -p "$OUTDIR/br/$(dirname "$F")"
    fi
    brotli -vfq 5 "$F" -o "$OUTDIR/br/$OUTFILE"
  done
  # scp -r "$OUTDIR" "$SSH_DEST"
else
  echo "No changed files!"
fi
