# bevy-game

Quell WIP repo

## Structure

This is a game built with Bevy. It uses a workspace configuration to split dependencies. Each crate should have a single distinct feature and should be represented in a bevy plugin. The final executable should be in the src/ directory and should involve as little configuration as possible.

## Building

This repository contains submodules. Make sure you call `git submodule init --recursive`.

If you're using WSL2, you should use the `cargo wsl` command below. This builds with MSVC, which can be a bit difficult to setup. If you don't want to do that, you can build with the windows GNU target, but you won't be able to debug. The linux build runs well on WSLg, but it has issues with cursor tracking, which makes the flycam unusable. Of course, if you're running Linux or MacOS natively, build for the default target and you should be fine. Just note that most people play games on Windows, and that you might have some issues without testing cross-platform compatibility.

Before you run `cargo wsl` you MUST have `libc++-6.dll` installed and available on your Windows system path. If you do not do this, the game will fail to start with no error message. Follow the instructions [here](https://www.msys2.org) to install MSYS2. We need the C++ stdlib installed so we can use `meshopt`.

## Environment

I highly recommend using [direnv](https://github.com/direnv/direnv) to manage environment variables. Any variables stored in .env.local will be git ignored. Example:

```sh
export RUST_LOG="warn,bevy_game=debug,worldgen=debug,"
export RUST_BACKTRACE=1
export CARGO_BUILD_TARGET="x86_64-pc-windows-msvc"
export DEBUG_LEVEL=0
export WINDBG_PORT=1234
export RENDERDOC_PORT=2345
```

## Development Scripts
All development scripts (if you are using direnv) are acessible through Cargo. Many of them require python to be installed and the virtual environment to be set up.

| script | description |
| --- | --- |
| cargo setup | Sets up all dependencies and activates the python venv. Will attempt to install system dependencies. Assumes you're on linux. |
| cargo chk | Runs lints with clippy and [bevy_lint](https://github.com/TheBevyFlock/bevy_cli), checks headers, etc. This is also run in CI. |
| cargo wsl | Builds the application for use on WSL. (May be outdated. Assumes a debian-based environment.) This uses the awesome [xwin](https://github.com/rust-cross/cargo-xwin) project for cross-compilation with the MSVC target. |
| cargo ci | Runs CI tests locally using nektos/act |
| cargo headers | Checks for headers. Part of cargo chk. |

## Features
By default, the crate builds with the `inspector` and `dev` feature flags enabled. This builds with bevy's dylib dependencies and the game inspector. For finished builds, you'll need to compile with `--no-default-features`.
