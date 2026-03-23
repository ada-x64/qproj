# How to work on this project

Thanks for your interest!

You should have a few prerequisite tools installed. Most importantly, you'll
need [mise](https://mise.jdx.dev/), an environment manager and task runner.

Once you install mise, run `mise install` and `mise run` to get all the
necessary tools and view the available actions.

## Why a monorepo?

These crates are all very closely related. It's likely that if you edit one,
you'll need to edit another. Additionally, they all use the same build tooling,
so overall it's more convenient to have them all in the same place.

```
qproj/
├── workspace/           # Cargo workspace root
│   ├── Cargo.toml       # Workspace manifest (members = ["crates/*"])
│   ├── Cargo.lock
│   ├── assets/
│   └── crates/
│       ├── app/             # Main game application
│       ├── screens/         # q_screens — screen management plugin
│       ├── term/            # q_term — in-game terminal plugin
│       └── test_harness/    # q_test_harness — shared test utilities
├── submod/bevy          # Bevy git submodule (reference only)
├── flake.nix            # Nix flake (dev shell, tools, bevy_lint)
├── justfile             # Task runner recipes
├── scripts/             # Task scripts invoked by justfile
├── .envrc               # direnv integration
├── .config/
│   └── nextest.toml     # Test runner config
├── .cargo/config.toml   # Cargo build settings (cranelift, mold)
├── deny.toml            # cargo-deny dependency policy
└── .github/workflows/   # CI definitions
```

All Cargo commands should target `workspace/Cargo.toml`, e.g.
`cargo build --manifest-path=./workspace/Cargo.toml`. The justfile recipes
handle this automatically.

## Setup

Install [Nix](https://nixos.org/download/) and
[direnv](https://direnv.net/docs/installation.html), then:

```sh
git clone --recurse-submodules https://github.com/ada-x64/qproj.git
cd qproj
direnv allow     # Activates the Nix dev shell automatically
just             # Lists all available tasks
```

Alternatively, without direnv:

```sh
nix develop      # Enter the dev shell manually
just             # Lists all available tasks
```

The Nix dev shell provides the full toolchain:

- **Rust nightly-2026-01-22** with cranelift, llvm-tools, and clippy
- **sccache** for compilation caching
- **mold** linker (Linux)
- **bevy_lint** (built from source)
- **cargo-nextest**, **cargo-deny**, **cargo-llvm-cov**
- **just** task runner

### System dependencies (Linux)

```sh
sudo apt-get install -y libasound2-dev libudev-dev libwayland-dev
```

## Building and running

| Command        | Description                              |
| -------------- | ---------------------------------------- |
| `just build`   | Build the workspace                      |
| `just play`    | Run the application                      |
| `just check`   | Lint with Clippy and bevy_lint           |
| `just deny`    | Check dependencies with cargo-deny       |
| `just test`    | Run tests via cargo-nextest              |
| `just coverage` | Generate test coverage report           |
| `just ci`      | Run CI locally with [act](https://github.com/nektos/act) |

You can pass arguments through to the underlying tools. For example:

```sh
just test r -p q_term              # Test a specific package
just build --release               # Release build
just clippy -p q_screens           # Lint a specific package
```

## Testing

Tests are handled
with [cargo-nextest](https://github.com/nextest-rs/nextest). Test coverage is
generated with [cargo-llvm-cov.](https://github.com/taiki-e/cargo-llvm-cov)
Generally, tests should be built out using
[q_test_harness.](./crates/test_harness) Read that documentation to learn
common testing patterns.

You can test CI locally by running `mise ci`. You can change which workflow and
which matrix parameters are set by using this sort of pattern:

```mise ci -W ./.github/workflows/ci.yml --matrix target:x86_64-unknown-linux-gnu```

## PRs and Commit Messages

PRs, when accepted, should be squashed into a single commit using [git
convention](https://www.conventionalcommits.org/en/v1.0.0/) for the message.
This is used to automatically generate changelogs when we publish.

PRs are expected to follow best practices. If you author a PR using an LLM,
please disclose that you have done so. All CI checks should pass.

### Running CI locally

```sh
just ci
# Or target a specific workflow/matrix:
just ci -W ./.github/workflows/ci.yml --matrix target:x86_64-unknown-linux-gnu
```

## Code style

### Conventions

- **Rust 2024 edition.** All crates use `edition = "2024"`.
- **Prelude pattern.** Each crate re-exports its public API through a
  `prelude` module. Internal imports use `pub(crate) use bevy::prelude::*`.
- **Documentation.** Public-facing crates should use `#![deny(missing_docs)]`.
- **Error handling.** Use `tiny_bail` for fallible Bevy systems and `thiserror`
  for typed errors.
- **Workspace lints.** Configured in `workspace/Cargo.toml` under
  `[workspace.lints]`. All crates inherit these with `[lints] workspace = true`.

### What the linters check

CI runs both `clippy` and `bevy_lint`. Make sure both pass before submitting:

```sh
just check
```

### Build profiles

| Profile       | Use case                        |
| ------------- | ------------------------------- |
| `dev`         | Local development (cranelift)   |
| `release`     | Optimized for size, stripped    |
| `dist`        | Distribution (max optimization) |
| `wasm-dev`    | WebAssembly development         |
| `server-dev`  | Server/headless development     |
| `android-dev` | Android development             |

Dev builds use the **cranelift** codegen backend and the **mold** linker for
fast iteration. Dependencies are built at `opt-level = 3` even in dev.

## PRs and commit messages

PRs are squash-merged into a single commit using
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format.
This powers automatic changelog generation.

Examples:

```
feat(screens): add transition animations
fix(term): correct line wrapping on resize
refactor: simplify test harness setup
```

All CI checks must pass. If you authored a PR using an LLM, please disclose
that.

## Publishing

Crates are published using
[cargo-smart-release.](https://github.com/crate-ci/cargo-release) Contributors
generally shouldn't be running this, so this documentation is mostly for me :p
Crates should have at least 80% test coverage before release. Currently there
are hooks to enforce this, so use your best judgement.
