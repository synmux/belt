# Prompt Collection

---

```plaintext
/lathe build me a guide to getting started with synmux/belt. You are in the code directory now, write to docs/. I don't know any Rust but I'm quite familar with higher level languages like Python and Ruby. Get me going. You can skip the 'setting up Rust' section, I'm ready to roll. use the /diagram-design and/or /archify skills to help you diagram things; I benefit greatly from diagrams. read `Cargo.toml` to see which libraries I'm using; in particular, `clap` for arg parsing, `ratatui` for TUI magic as needed. plenty of others, see the `README.md`. as it stands right now, it's pretty much boilerplate with deps.

planned features:

### `belt decent`

- Chat
  - automatic kex
  - shared wrapper key for privacy
  - file sending, see later
  - use webrtc for comms and DHTs for hello?
- File send/receive
  - How does Magic Wormhole work?
- Collaborative text editor
  - Maybe just handle sync for the file, and trust the user's preferred editor to notice changes?
  - A CRDT seems to be the key piece
- Optionally backend on a shared MQTT broker.
  - Or maybe use one for hello?
  - Make sure to use TLS and pick one which does TLS over WebSockets
  - Public brokers
    - `broker.emqx.io`
    - `mqtt.eclipseprojects.io`
    - `test.mosquitto.org`
    - `broker.hivemq.com`
    - `mqtt.flespi.io`
    - `mqtt.dioty.co`
- What other cool decentralised stuff could we implement?

### `belt tag`

Generate a readable string correlated to a hash value over a file. Used for version specifiers.

Uses the BIP39 wordlist.

- `belt tag [FILE]` - hash the file, generate the tag.
- `belt tag` - generate a random tag.

### `belt mise`

Helpers for `mise`.

- `belt mise [ECOSYSTEM] [PACKAGE] [ARGS]` - add `PACKAGE` in `ECOSYSTEM` to the global `mise.toml`. `chezmoi` aware with `chezmoi = true` in the config.

## Configuration

### Global

`chezmoi` - Boolean. Integrate with `chezmoi` for file ops.
```

---
