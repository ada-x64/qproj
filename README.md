# Quell

This is a game project written in [Bevy](https://bevyengine.org) featuring procedural terrain generation. You can read more about it [on my blog.](https://cubething.dev/qproj/general-introduction)

## Getting Started

This module uses submodules. Clone like this:

```bash
git clone --recursive-submodules https://github.com/ada-x64/qproj
```

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
