# `belt`

[![CI](https://github.com/synmux/belt/actions/workflows/ci.yml/badge.svg)](https://github.com/synmux/belt/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A CLI toolbelt, written in Rust.

> **Status:** pre-alpha scaffold. The binary currently prints `Hello, world!` and nothing else.
> `belt` is being rewritten from scratch. The earlier Python implementation lives on the
> [`python`](https://github.com/synmux/belt/tree/python) branch and is no longer published.

## Requirements

- Rust `1.98.1` (pinned in [`mise.toml`](mise.toml); `mise install` will fetch it)
- Optionally [`trunk`](https://docs.trunk.io/cli) for the full lint suite
- Optionally Docker for the container build

## Building

```shell
cargo build --release
./target/release/belt
```

Or via Docker:

```shell
docker build --tag belt .
docker run --rm belt
```

## Development

```shell
cargo fmt --all
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
trunk check
```

The same checks run in CI on every push and pull request. Dependabot keeps Cargo, Docker and
GitHub Actions dependencies current.

Guidance for AI coding agents lives in [`AGENTS.md`](AGENTS.md) (`CLAUDE.md` is a symlink to it).

## Licence

MIT. See [`LICENSE`](LICENSE).
