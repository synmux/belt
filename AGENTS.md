# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`AGENTS.md` is the canonical file; `CLAUDE.md` is a symlink to it. Edit `AGENTS.md`.

## What this is

`belt` is a Rust CLI toolbelt (crate `belt-cli`, binary `belt`). It is a ground-up rewrite of a
previous Python package of the same name. The Python implementation is preserved on the `python`
branch (`git show python:src/belt/...`, or `git worktree add ../belt-python python`) and is the
reference for what to port. It shipped `audio`, `crypt`, `dns`, `domain` and `tls` command groups
and a YAML config at `~/.config/belt/config.yaml`; none of that has been ported yet.

Bare `belt` shows help and exits successfully (also the Docker smoke test). `belt pw [MIN_LENGTH]`
prints a single five-word password and a newline to stdout. Its minimum is exclusive: the default
of 18 requires at least 19 characters, the shortest any password can be, so it rejects nothing and
costs no entropy. A non-negative argument replaces the default. Too-short candidates are
regenerated silently. Keep warnings and errors on stderr; never log a rejected password.

## Toolchain

- `mise.toml` pins the local Rust version. Bump it together with `Dockerfile`, `.github/workflows/ci.yml`
  and the `clippy`/`rustfmt` entries in `.trunk/trunk.yaml`.
- `mise` manages the toolchain (`mise install`). It also sets `BELT_MISE_ACTIVE=true`, loads `.env`
  if present, and wraps `cargo` in `mbx` (mr-boxington) for cached builds. `mise run bump` upgrades
  the pinned tools; it reads a GitHub token from 1Password through `op run`.
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

- `clap` with `derive`, `env`, `unicode`, `wrap_help`, `string`, `suggestions`, `error-context`,
  `help`, `usage` and `color` features. Use the derive API.
- `bip39` supplies the English dictionary for `pw`; `rand` supplies its OS-seeded cryptographic RNG.
- Everything else is groundwork that no code uses yet, so cargo reports it as unused dependencies.
  Those manifest warnings are expected and don't fail the clippy gate.
  - `ratatui` and `crossterm` for a future TUI. `crossterm` has `osc52` enabled for clipboard writes
    over the terminal.
  - `clap_complete` for shell completions and `clio` for file, stdin and stdout arguments.
  - `blake2` and `ur` (Bytewords) for the `belt tag` command built in `docs/learning`.
  - `shadow-rs` for build metadata and the `clap_mangen` build dependency for man pages. Both need
    a `build.rs`, which doesn't exist yet.
  - `tera` and `tera-contrib` for template rendering.

## Layout

- `src/main.rs` parses and dispatches commands for the single `belt` binary (`default-run = "belt"`).
- `src/pw.rs` handles password generation, including the dictionary-derived entropy thresholds and
  the silent retry loop. Its unit tests sit alongside it; `tests/pw_cli.rs` checks process status
  and both streams.
- `Dockerfile` is a two-stage build (a slim Bookworm Rust builder then `debian:bookworm-slim`)
  running as UID 10001. It copies only `Cargo.toml`, `Cargo.lock` and `src/`, so any new top-level
  directory the build needs must be added there and removed from `.dockerignore`. The `apt`
  package versions are pinned; `hadolint` requires that.
- `.github/dependabot.yml` covers cargo, docker and github-actions with a seven-day cooldown.
- `docs/learning/` is a tutorial series that builds further commands step by step. The reader
  applies its code, so none of it is in `src/`.
- `.serena/` is Serena's project config and is excluded from trunk.

## Conventions

- Commits: Conventional Commits with a leading GitMoji, multi-line body with full detail. See
  `git log` for the house style.
- UK English in prose, comments and user-facing strings.
- Descriptive identifiers only; no single-letter names, including in closures and loops.
- Four-space indentation for Rust per `.editorconfig`; two spaces for YAML, TOML and Markdown.
