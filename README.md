# bevy-game

Quell WIP repo

## Structure

This is a game built with Bevy. It uses a workspace configuration to split dependencies. Each crate should have a single distinct feature and should be represented in a bevy plugin. The final executable should be in the src/ directory and should involve as little configuration as possible.

## Building

This repository contains submodules. Make sure you call `git submodule init --recursive`.

If you're using WSL2, you should use the `cargo wsl` command below. This builds with MSVC, which can be a bit difficult to setup. If you don't want to do that, you can build with the windows GNU target, but you won't be able to debug. The linux build runs well on WSLg, but it has issues with cursor tracking, which makes the flycam unusable. Of course, if you're running Linux or MacOS natively, build for the default target and you should be fine. Just note that most people play games on Windows, and that you might have some issues without testing cross-platform compatibility.

Before you run `cargo wsl` you MUST have `libc++-6.dll` installed and available on your Windows system path. If you do not do this, the game will fail to start with no error message. Follow the instructions [here](https://www.msys2.org) to install MSYS2. We need the C++ stdlib installed so we can use `meshopt`.