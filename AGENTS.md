# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`AGENTS.md` is the canonical file; `CLAUDE.md` is a symlink to it. Edit `AGENTS.md`.

## What this is

`belt` is a Rust CLI toolbelt (crate `belt-cli`, binary `belt`). It is a ground-up rewrite of a
previous Python package of the same name, which was removed in commit `fab35b7`. The Python version
shipped `audio`, `crypt`, `dns`, `domain` and `tls` command groups and a YAML config at
`~/.config/belt/config.yaml`; none of that has been ported yet. The current `src/main.rs` is a
placeholder that prints "Hello, world!".

## Toolchain

- Rust `1.98.1` is pinned in `mise.toml`, `Dockerfile` and `.github/workflows/ci.yml`. Bump all
  three together.
- Edition `2024` in `Cargo.toml`. Note that `.trunk/configs/.rustfmt.toml` says `edition = "2021"`;
  trunk's rustfmt run uses that file, while `cargo fmt` reads `Cargo.toml`. Prefer `cargo fmt`.
- `mise` manages the toolchain (`mise install`). It also sets `BELT_MISE_ACTIVE=true` and loads
  `.env` if present.
- `trunk` runs the linters listed in `.trunk/trunk.yaml` and installs `trunk-fmt-pre-commit` and
  `trunk-check-pre-push` git hooks. Run `trunk check` before pushing if you want to see what the
  hook will complain about.

## Commands

```shell
cargo build                      # debug build
cargo run -- <args>              # run the binary
cargo test --locked --all-features
cargo test <test_name>           # single test by substring match
cargo test <module>::            # all tests in a module
cargo test -- --nocapture        # show stdout from tests
cargo fmt --all -- --check       # formatting gate (CI)
cargo clippy --locked --all-targets --all-features -- -D warnings   # lint gate (CI)
cargo build --locked --release --bin belt
trunk check                      # everything in .trunk/trunk.yaml (clippy, rustfmt, hadolint, yamllint, etc.)
trunk fmt                        # auto-format everything trunk knows about
docker build --tag belt:dev . && docker run --rm belt:dev
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy with `-D warnings`, test, a release build, and a
Docker build plus smoke run, all with `--locked`. Clippy warnings fail the build, so fix them rather
than suppressing them.

## Dependencies already in `Cargo.toml`

- `clap` with `derive`, `env`, `suggestions`, `wrap_help` and colour features. Use the derive API.
- `ratatui` and `crossterm` for any TUI surface. `crossterm` has `osc52` enabled for clipboard
  writes over the terminal.

Nothing uses these yet; they are pre-wired for the port.

## Layout

- `src/main.rs` is the only source file. Cargo.toml declares a single `[[bin]]` named `belt` at that
  path, with `default-run = "belt"`.
- `Dockerfile` is a two-stage build (`rust:1.98.1-slim-bookworm` then `debian:bookworm-slim`)
  running as UID 10001. It copies only `Cargo.toml`, `Cargo.lock` and `src/`, so any new top-level
  directory the build needs must be added there and removed from `.dockerignore`. The `apt`
  package versions are pinned; `hadolint` requires that.
- `.github/dependabot.yml` covers cargo, docker and github-actions with a seven-day cooldown.
- `.serena/` is Serena's project config and is excluded from trunk.

## Conventions

- Commits: Conventional Commits with a leading GitMoji, multi-line body with full detail. See
  `git log` for the house style.
- UK English in prose, comments and user-facing strings.
- Descriptive identifiers only; no single-letter names, including in closures and loops.
- Four-space indentation for Rust per `.editorconfig`; two spaces for YAML, TOML and Markdown.
