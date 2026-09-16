# belt: project overview (as of 2026-09-16)

- Rust CLI toolbelt, crate `belt-cli`, binary `belt`, edition 2024, Rust 1.98.1 pinned in `mise.toml`, `Dockerfile`, and CI.
- Ground-up rewrite of the Python package. The old Python implementation is preserved on the `python` branch (local and `origin/python`); use it as the porting reference. It had `audio`, `crypt`, `dns`, `domain`, `tls` command groups and config at `~/.config/belt/config.yaml`. Nothing ported yet; `src/main.rs` is Hello World.
- Pre-wired deps: `clap` (derive), `ratatui`, `crossterm` (osc52). Unused so far.
- Canonical agent guidance is `AGENTS.md`; `CLAUDE.md` is a symlink to it. `README.md` is boilerplate.
- CI gate: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, release build, Docker build + smoke run, all `--locked`. `trunk check` runs the wider linter set and installs pre-commit/pre-push hooks.
- Known quirk: `.trunk/configs/.rustfmt.toml` says edition 2021 vs Cargo.toml 2024. Prefer `cargo fmt`.
- `mise.toml` uses multi-line inline tables; an older commit message claimed this was invalid TOML but `mise current rust` parses it fine.
