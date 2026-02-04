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


## Publishing

Crates are published using
[cargo-smart-release.](https://github.com/crate-ci/cargo-release) Contributors
generally shouldn't be running this, so this documentation is mostly for me :p
Crates should have at least 80% test coverage before release. Currently there
are hooks to enforce this, so use your best judgement.
