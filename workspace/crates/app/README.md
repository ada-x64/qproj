app

## Goals

Meta level goals:

- Lightweight runtime framework.
- CLI-based creation of templates. (Think `ng`.)
- Well-tested feature set

Features:

- [x] Screen transitions (with [q_screens](../screens))
  - [x] Asset loading (with `bevy_asset_loader`).
  - [x] Asset unloading
  - [x] Persist _only_ those entities marked `Persistent`.
  - [x] Propogation of persistence property to entity children.
- [x] Service-oriented framework
  - [x] Screen-scoped systems
- [x] Devex
  - [x] Lightweight console (with [q_cmd_prompt](../cmd_prompt))
    - [ ] Screen integration
    - [ ] Utility commands
  - [x] CLI
    - [x] Template scaffolding
      - [x] Screens
      - [x] Services

## About the bird

"Tufted titmice nest in a hole in a tree, either a natural cavity, a human-made nest box, or sometimes an old woodpecker nest. They line the nest with soft materials, sometimes plucking hair from live mammals to use as material, a behavior known as kleptotrichy. If they find snakeskin sheddings, they may incorporate pieces into their nest." ([wikipedia](https://en.wikipedia.org/wiki/Tufted_titmouse))
