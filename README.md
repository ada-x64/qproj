# bevy-game

Quell WIP repo

## Running with Windows GNU or WSLg

If you're using WSL2, you should use the `cargo wsl` command below. This builds with MSVC, which can be a bit difficult to setup. If you don't want to do that, you can build with the windows GNU target, but you won't be able to debug. The linux build runs well on WSLg, but it has issues with cursor tracking, which makes the flycam unusable. Of course, if you're running Linux or MacOS natively, build for the default target and you should be fine. Just note that most people play games on Windows, and that you might have some issues without testing cross-platform compatibility.

## Building for MSVC on WSL2 (Recommended)

### Using direnv for cargo commands

Install direnv with your package manager and follow the instructions [here](https://direnv.net/docs/hook.html) to get it set up with your shell. The `.envrc` file here just adds the scripts folder to your path so you can run them as cargo commands. If you don't want to do that, you can just run the scripts manually.

### Building and running

```sh
cargo setup
cargo wsl --help
```

I highly recommend building with MSVC debug tools because you will be able to run the application as if it was build on a Windows machine and use the Windows-native debugging tools. If you want to use renderdoc from the command line, you will need to add it to your Windows $PATH.
