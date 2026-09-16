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

## Dependencies

Mostly a way to remind myself why I've pulled something in.

### `clap`, `ratatui`, `crossterm`

CLI arg parsing and TUI.

### `clap_complete`, `clio`

Extra goo for `clap`.

### `shadow-rs`

Pull crate metadata into the live code.

### `blake2`

Generate bytes for `ur`.

### `ur`

Has `ur::bytewords` for the `tag` function.

### `bip39`

For use if `bytewords` has problems.

## Licence

MIT. See [`LICENSE`](LICENSE).
