# belt: project overview (as of 2026-10-12)

- Rust CLI toolbelt, crate `belt-cli`, binary `belt`, edition 2024, Rust 1.99.0 pinned in `mise.toml`, `Dockerfile`, and CI.
- Ground-up rewrite of the Python package. The old Python implementation is preserved on the `python` branch (local and `origin/python`); use it as the porting reference. It had `audio`, `crypt`, `dns`, `domain`, `tls` command groups and config at `~/.config/belt/config.yaml`. Nothing ported yet; `src/main.rs` is Hello World.
- Pre-wired deps: `clap` (derive), `ratatui`, `crossterm` (osc52). Unused so far.
- Canonical agent guidance is `AGENTS.md`; `CLAUDE.md` is a symlink to it. `README.md` is boilerplate.
- CI gate: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, release build, Docker build + smoke run, all `--locked`. `trunk check` runs the wider linter set and installs pre-commit/pre-push hooks.
- Known quirk: `.trunk/configs/.rustfmt.toml` says edition 2021 vs Cargo.toml 2024. Prefer `cargo fmt`.
- `mise.toml` uses multi-line inline tables; an older commit message claimed this was invalid TOML but `mise current rust` parses it fine.
- Tutorial series `getting-started` lives in `~/.lathe/tutorials/getting-started/` (lathe) and is mirrored to `docs/getting-started/`. Part 1 builds `belt tag` (clap + clio + blake2 + ur Bytewords); Part 2 adds config via `confy` 2.0.0 (TOML at `~/.config/belt/config.toml`, `--config`/`BELT_CONFIG`, `belt config path|init`); Part 3 adds `belt mise add TOOL@VERSION` (`toml_edit` 0.25.17 format-preserving edit of `~/.config/mise/config.toml`, `[mise] chezmoi = true` routes the edit through `chezmoi source-path`/`decrypt`/`encrypt`/`apply`, line-based edit for `.tmpl` sources, `etcetera` 0.10.0 as a direct dep). The code from the parts is NOT committed to `src/`; `src/main.rs` is still Hello World until the reader applies the parts.
- Part 2's "What's next" promised a `BeltError` enum and random tags via `rand`; Part 3 deferred both. Part 4 should start there.
- Research notes in Basic Memory under projects/belt: "Rust config crate survey for belt (October 2026)" and "chezmoi integration for belt mise (October 2026)" (verified chezmoi 2.73.0 behaviour, toml_edit decor gotcha, mise global config facts).
- The owner's real mise config is chezmoi-managed as `dot_config/mise/config.toml.tmpl` (template, not encrypted), with `[tools]` once and `{{- if eq .chezmoi.os "darwin" }}` blocks nested below it. Never run chezmoi write commands against it from a session; test with `HOME=/tmp/...` isolation.
