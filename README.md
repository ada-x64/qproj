# Quell

This is a game project written in [Bevy](https://bevyengine.org) featuring procedural terrain generation. You can read more about it [on my blog.](https://cubething.dev/qproj/general-introduction)

## Structure

This project uses a workspace configuration to split dependencies. Each crate should have a single distinct feature and should be represented in a bevy plugin. The final executable should be in the src/ directory and should involve as little configuration as possible.
Read more about it [on my blog.](https://www.cubething.dev/qproj/architecture-1---plugin-hierarchies)

## Building

### System deps

- UNIX-like environment (I use Ubuntu WSL and Manjaro Linux)
- [just](https://github.com/casey/just)
- python3
- bash

Run `just setup` to install all the build dependencies.

### WSL2

Before you run `cargo wsl` you MUST have `libc++-6.dll` installed and available on your Windows system path. If you do not do this, the game will fail to start with no error message. Follow the instructions [here](https://www.msys2.org) to install MSYS2. We need the C++ stdlib installed so we can use `meshopt`.

## Features
By default, the crate builds with the `inspector` and `dev` feature flags enabled. This builds with bevy's dylib dependencies and the game inspector. For finished builds, you'll need to compile with `--no-default-features`.
