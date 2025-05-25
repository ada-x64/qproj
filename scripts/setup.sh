#!/bin/bash

if [ ! -d .venv ]; then
  echo "Setting up virtual environment..."
  python3 -m venv .venv
  . .venv/bin/activate
fi
python3 -m pip install -r requirements.txt

shift
.venv/bin/python3 scripts/cargo-setup.py "$@"
