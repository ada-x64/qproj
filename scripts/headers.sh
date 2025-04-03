#!/bin/bash

status=0

function set_header() {
  h="\
$p         •\n\
$p ┏┓┏┓┏┓┏┓┓\n\
$p ┗┫┣┛┛ ┗┛┃\n\
$p--┗┛-----┛------------------------------------------ (c) 2025 contributors ---"
}

function doit() {
  set_header 
  for f in $files; do
    content=$(head "$f" -n 4)
    diff -y <(echo -e "$content") <(echo -e "$h") > /dev/null
    if [ "$?" -eq "0" ]; then
      echo "OK: $f"
    else 
      status=1
      echo "EDITING: $f"
      sed -si -e "1i\\$h" "$f"
    fi
  done
}

p="//"
files=$(find ./ -not -path "./target/*" -not -path "**/gen/*" -name "*.rs")
doit

p="# "
files=$(find ./ -not -path "./target/*" -name "*.toml")
doit

exit $status