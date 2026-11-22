# `belt`

[![CI](https://github.com/synmux/belt/actions/workflows/ci.yml/badge.svg)](https://github.com/synmux/belt/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A CLI toolbelt, written in Rust. Its `pw` command generates four-word passwords.

> **Status:** pre-alpha. `belt` is being rewritten from scratch in Rust.

## Requirements

- Rust (version pinned in [`mise.toml`](mise.toml); `mise install` will fetch it)
- Optionally [`trunk`](https://docs.trunk.io/cli) for the full lint suite
- Optionally Docker for the container build

## Building

```shell
cargo build --locked --release --bin belt
./target/release/belt
```

Or via Docker:

```shell
docker build --tag belt .
docker run --rm belt
```

## Generating a password

Running `belt` without arguments shows the available commands. Use `belt pw [MIN_LENGTH]` to print
one password followed by a newline. The password must be **strictly longer** than `MIN_LENGTH`:

```shell
belt pw           # Default: longer than 16 characters (at least 17)
belt pw 12        # Override: longer than 12 characters (at least 13)
belt pw --help    # Show usage without generating a password
```

Each password uses four words drawn independently from the English BIP39 dictionary; words may
repeat. Between one and three randomly chosen words are uppercase and the rest are lowercase.
The three gaps each contain one ASCII digit (`0`–`9`) or one symbol from `&+.,/-=?@#^%`; both
types occur in every password. All random choices use a cryptographically secure generator seeded
from the operating system.

Longer minima exclude more word combinations. `belt` checks the fraction excluded by the requested
length before generating anything. With the current dictionary:

| `MIN_LENGTH` | Behaviour                                                                                       |
| ------------ | ----------------------------------------------------------------------------------------------- |
| 0–25         | Generate without an entropy warning.                                                            |
| 26–28        | More than 70% of combinations are too short. Warn once about restricted entropy, then generate. |
| 29 or more   | More than 95% are too short. Refuse the request (including `belt pw 32`) and exit non-zero.     |

Each rejected candidate produces a retry notice without revealing the candidate. Warnings, retry
notices and errors go to stderr, so stdout contains only the accepted password and its newline,
or nothing on failure.

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

### `bip39`, `rand`

`bip39` supplies the English word list for `pw` and is a possible fallback for `bytewords`.
`rand` provides OS-seeded cryptographic randomness for password generation.

## Licence

MIT. See [`LICENSE`](LICENSE).
