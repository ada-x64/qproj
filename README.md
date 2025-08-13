<div align="center">
<a href="https://cubething.dev/qproj/general-introduction"><img src="https://cubething.dev/static/qproj-quail-logo.webp" height=128 alt="Illustration of a quail next to text, 'qproj'" title="qproj logo" /></a>
<br/>
<a href="https://github.com/ada-x64/qproj">main project </a>
|
<a href="https://github.com/ada-x64/q_service"> q_service  </a>
<br/>
<a href="https://wraithcastle.com" style="font-size: small">illustration by wraithcastle</a>
</div>

---

This is a game project written on top of [Bevy.](https://bevyengine.org)
Read more [on my blog.](https://cubething.dev/qproj/general-introduction)

## Features

- Chunked and optimized procedural 3D terrain generation
- An event-driven, service-oriented architecture built on top of [q_service](https://github.com/ada-x64/q_service)
- A custom editor
- ... more

## Getting Started

This repo uses submodules. Clone like this:

```bash
git clone --recursive-submodules https://github.com/ada-x64/qproj
```

### System deps

- UNIX-like environment (I use Ubuntu WSL and Manjaro Linux)
- [just](https://github.com/casey/just)
- python3
- bash

Run `just setup` to install all the build dependencies.

### Structure

This project uses a workspace configuration to split dependencies. Each crate
should have a single distinct feature and should be represented in a bevy
plugin. The final executable should be in the src/ directory and should involve
as little configuration as possible. Read more about it [on my
blog.](https://www.cubething.dev/qproj/architecture-1---plugin-hierarchies)

## License

This project is available under the terms of either the [Apache 2.0
license](./LICENSE-APACHE.txt) or the [MIT license](./LICENSE-MIT.txt).
