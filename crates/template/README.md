<div align="center">
<img src="../../.doc/tfw.png" height=300 alt="Illustration of a bluejay with text, 'tfw - bevy template framework'" title="tfw logo" />
</div>

This is a template-based template for a bevy project. It is a simplification of the work done for [q_service.](https://github.com/ada-x64/q_service)

## Goals

Meta level goals:

- Lightweight runtime framework.
- CLI-based creation of templates. (Think `ng`.)
- Well-tested feature set

Features:

- [x] Screen transitions
  - [x] Asset loading (with `bevy_asset_loader`).
  - [ ] Asset unloading
  - [x] Persist _only_ those entities marked `Persistent`.
  - [x] Propogation of persistence property to entity children.
- [x] Service-oriented framework
  - [x] Screen-scoped systems
- [ ] Devex
  - [ ] Lightweight console (with [bevy_ui_console](https://github.com/ada-x64/bevy_command_prompt))
  - [x] CLI
    - [x] Template scaffolding
      - [x] Screens
      - [x] Services

## System deps

- UNIX-like environment (I use Manjaro Linux)
- [mise](https://mise.jdx.dev)

Call `mise run` to see a list of all the available commands.

### Developing over headless SSH

The mise scripts allow you to develop over SSH. It assumes you have symmetric
SSH access, and that both machines can use rsync. By default, builds will
proceed with -Fdev and -Fdylib, but over ssh we do not dynamically link.
This increases build times but allows us to sync far faster.

## License

This project is available under the terms of either the [Apache 2.0
license](./LICENSE-APACHE.txt) or the [MIT license](./LICENSE-MIT.txt).
