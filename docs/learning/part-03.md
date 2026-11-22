# belt, Part 3: errors with names, random tags, and `belt mise`

Every error `belt` can produce after Part 2 is a `Box<dyn Error>`, and most of them started life as a `format!` string. The one for a bad config file reads `error: could not read bad-key.toml: Bad TOML data: TOML parse error at line 1, column 1`, three messages joined by colons by a helper that walks the error's `source()` chain. It is readable. It is also the only thing the program knows about the failure. `run` can return any type that implements `Error`, including a `String`, so the compiler has no list of what can go wrong, and a test that wants to check for one failure has to search the message for a substring and hope nobody rewords it.

This part replaces that with one `enum`, `BeltError`, with one variant per failure, each carrying the path or the command or the value that the message needs. `Display` is written once, in the first large trait implementation of this series, and the string-walking helper from Part 2 is deleted. Then the `None` arm of `belt tag`, which has returned "not implemented yet" since Part 1, gets real random bytes.

Both of those are groundwork for `belt mise`. `mise use --global node@22` writes to `~/.config/mise/config.toml`, and if chezmoi manages that file, the next `chezmoi status` prints `MM .config/mise/config.toml`: modified since chezmoi last wrote it, and about to be modified again. If the source is a template, mise's write went to the rendered output, the one file nobody is supposed to edit. If the source is encrypted, mise never saw it at all. `belt mise add node@22` changes `tools.node` in the file chezmoi owns, through chezmoi's own decryption when it has to, and leaves every other byte alone. The helper that talks to chezmoi has three distinct ways to fail, which is where the new error type earns its keep.

> [!RECALL]
> Before you continue: Part 2 put `#[serde(default)]` on all three config structs, not only on `Config`. If the attribute were on `Config` alone, what happens to a file that contains `[tag]` with no `words` key under it, and why?

By the end of this part, you'll have a `src/error.rs` with twelve named failures, a `main` that prints each cause on its own line, and a `belt tag` that prints four random words when given no file. You'll also have `belt mise add TOOL@VERSION`, which edits a plain file, an encrypted file, or an encrypted template through chezmoi's own commands. Nine new tests bring the total to fifteen; the Checkpoint runs the whole thing against a throwaway chezmoi with a real age key.

## What you'll build

`belt tag` with no argument prints something like `roof-horn-very-road`, different every run. A config file with `words = 100` fails with `error: tag.words must be 1 to 64, got 100`, and the test for it matches the enum variant instead of the text. `belt mise add node@22` sets `tools.node = "22"` in mise's global config; on a file containing `node   = "20" # LTS`, the result is `node   = "22" # LTS`, spacing and comment intact. With `chezmoi = true` under `[mise]` in `~/.config/belt/config.toml`, the same command runs `chezmoi source-path`, reads the filename to learn whether the source is encrypted or a template or both, edits accordingly, and finishes with `chezmoi apply` on that one target. The code is a new `src/error.rs` of 122 lines, a new `src/mise.rs` of about 230 lines plus 80 lines of tests, 15 lines in `src/tag.rs`, and small edits to `src/config.rs` and `src/main.rs` where `Box<dyn Error>` used to be.

## Prerequisites

- Part 2 finished: `cargo test` reports six passing tests and `belt config init` writes a file.
- `chezmoi` 2.73.0 on your `PATH` for the chezmoi sections and for the Checkpoint. `mise use --global chezmoi@latest` or `brew install chezmoi` both work. The tests, the error type, random tags, and the direct mise edit need neither chezmoi nor mise.
- No `age` binary is required. chezmoi has [built-in age encryption](https://www.chezmoi.io/user-guide/encryption/age/#builtin-age-encryption) that it uses when `age` is not on `PATH`; if `age` is installed, chezmoi calls it instead, and the output is the same armoured format.
- Nothing about Go templates beyond recognising `{{ ... }}` as one.

## Every failure gets a name

Create `src/error.rs`. The enum comes first, and it is longer than any type so far because it lists failures from the whole binary, including the `belt mise` ones you have not written yet. Read the doc comments; each one is the sentence the variant replaces.

```rust
//! One error type for the whole binary: every way `belt` can fail has a name.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::string::FromUtf8Error;

use confy::ConfyError;

use crate::config::MAX_TAG_WORDS;

/// Everything `run` can return instead of `Ok(())`.
#[derive(Debug)]
pub enum BeltError {
    /// `confy` could not work out where `~/.config/belt/config.toml` is.
    ConfigPath(ConfyError),
    /// The configuration file could not be read, parsed, or written.
    Config { path: PathBuf, source: ConfyError },
    /// `belt config init` without `--force` on a file that already exists.
    ConfigExists { path: PathBuf },
    /// `tag.words` parsed fine but is outside 1 to 64.
    WordsOutOfRange { words: usize },
    /// `belt tag` could not read its input.
    Input(io::Error),
    /// `belt mise add` was given something other than `TOOL@VERSION`.
    ToolSpec { spec: String },
    /// `etcetera` could not find a home directory for `~/.config/mise`.
    NoHome(etcetera::HomeDirError),
    /// A plain file read or write failed.
    File { path: PathBuf, source: io::Error },
    /// `toml_edit` rejected the file.
    Toml {
        path: PathBuf,
        source: toml_edit::TomlError,
    },
    /// A key in the file has the wrong shape, such as `tools = "nope"`.
    Shape { key: String, expected: &'static str },
    /// The `chezmoi` binary could not be started.
    ChezmoiMissing(io::Error),
    /// A `chezmoi` command exited non-zero; `stderr` is what it said.
    ChezmoiFailed {
        command: String,
        status: ExitStatus,
        stderr: String,
    },
    /// A `chezmoi` command printed bytes that are not UTF-8.
    ChezmoiOutput {
        command: String,
        source: FromUtf8Error,
    },
}
```

Two kinds of variant are mixed here. `ConfigExists`, `WordsOutOfRange`, `ToolSpec`, `Shape` and `ChezmoiFailed` are **leaf** errors: `belt` detected the problem itself and the fields are the facts the message needs. The others **wrap** an error from another crate or from the standard library in a field named `source`, and keep the context that error lacks, usually a path. `io::Error` on its own says `No such file or directory (os error 2)` and does not name the file; `File { path, source }` is that error plus the answer. `#[derive(Debug)]` is not decoration: the `Error` trait requires `Debug`, and the derive is how `assert!(matches!(...))` in the tests prints a value that did not match.

The `etcetera` and `toml_edit` names resolve after the `cargo add` in a later section; until then this file does not compile, which is fine, because nothing uses it yet.

Now the promise from Part 2: a bigger trait implementation. [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) is the trait behind `{}` in a format string, and it has one method:

```rust
impl fmt::Display for BeltError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigPath(_) => {
                write!(formatter, "could not work out where the configuration file lives")
            }
            Self::Config { path, .. } => {
                write!(formatter, "configuration file {}", path.display())
            }
            Self::ConfigExists { path } => write!(
                formatter,
                "{} already exists; pass --force to overwrite it",
                path.display()
            ),
            Self::WordsOutOfRange { words } => {
                write!(formatter, "tag.words must be 1 to {MAX_TAG_WORDS}, got {words}")
            }
            Self::Input(_) => write!(formatter, "could not read the input"),
            Self::ToolSpec { spec } => {
                write!(formatter, "expected TOOL@VERSION, for example node@22, got {spec:?}")
            }
            Self::NoHome(_) => write!(formatter, "could not find your home directory"),
            Self::File { path, .. } => write!(formatter, "{}", path.display()),
            Self::Toml { path, .. } => write!(formatter, "could not edit {}", path.display()),
            Self::Shape { key, expected } => write!(formatter, "`{key}` is not {expected}"),
            Self::ChezmoiMissing(_) => write!(formatter, "could not run chezmoi"),
            Self::ChezmoiFailed {
                command,
                status,
                stderr,
            } => write!(formatter, "chezmoi {command}: {status}: {stderr}"),
            Self::ChezmoiOutput { command, .. } => {
                write!(formatter, "chezmoi {command} printed something that is not UTF-8")
            }
        }
    }
}
```

`write!` with a `Formatter` as its first argument is `println!` aimed at whoever asked for the text, and it returns the `fmt::Result` the method has to return, so each arm is one expression. `Self::` is the enum, so the arms do not repeat `BeltError::` thirteen times. The `..` in `Self::Config { path, .. }` ignores the `source` field; the wrapped error is not printed here, and the next listing says where it goes instead.

Here is the wrong version, which is any version with an arm missing. Delete the `Input` arm and the compiler says:

```text
error[E0004]: non-exhaustive patterns: `&BeltError::Input(_)` not covered
```

This exhaustiveness check is the reason to prefer an enum over a bag of strings: when a later part adds a variant, every `match` on `BeltError` in the codebase fails to compile until it says what to do with the new case. A new `format!` string would have added a failure that nothing else in the program knew about.

The [`Error`](https://doc.rust-lang.org/std/error/trait.Error.html) trait itself has no required methods, so the implementation could be empty. It is not, because [`source`](https://doc.rust-lang.org/std/error/trait.Error.html#method.source) is the hook that the wrapped errors hang from:

```rust
impl Error for BeltError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ConfigPath(source) | Self::Config { source, .. } => Some(source),
            Self::Input(source) | Self::ChezmoiMissing(source) | Self::File { source, .. } => {
                Some(source)
            }
            Self::NoHome(source) => Some(source),
            Self::Toml { source, .. } => Some(source),
            Self::ChezmoiOutput { source, .. } => Some(source),
            Self::ConfigExists { .. }
            | Self::WordsOutOfRange { .. }
            | Self::ToolSpec { .. }
            | Self::Shape { .. }
            | Self::ChezmoiFailed { .. } => None,
        }
    }
}
```

The `|` between patterns is an **or-pattern**, and the grouping is not cosmetic. Every alternative in an or-pattern has to bind `source` with the same type, so the two `ConfyError` variants share an arm, the three `io::Error` variants share another, and the three remaining wrappers each get their own. The leaf variants return `None` together. The compiler checks this list for completeness too.

That is the whole file. The reason it can be shorter than Part 2's `describe` helper would suggest is in `src/main.rs`, where the chain-walking moves to, in the one place that prints errors. Replace `main`:

```rust
fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        let mut cause = std::error::Error::source(&error);
        while let Some(inner) = cause {
            eprintln!("  caused by: {inner}");
            cause = inner.source();
        }
        std::process::exit(1);
    }
}

fn run() -> Result<(), BeltError> {
```

and add `mod error;` after `mod config;` and `use error::BeltError;` after `use config::Config;`. Part 2 promised the `describe` loop would go, and it has gone from `config.rs`, where it was flattening three messages into one line with colons. The four lines that replaced it print each cause on its own line, once, for every error in the program. The same bad file from Part 2 now reports:

```text
error: configuration file bad-key.toml
  caused by: Bad TOML data
  caused by: TOML parse error at line 1, column 1
  |
1 | colour = "red"
  | ^^^^^^
unknown field `colour`, expected one of `tag`, `decent`, `mise`
```

The first line is `BeltError::Config`'s `Display`, with the path. The second is `ConfyError`'s. The third is the TOML parser's, reached through two `source()` hops. Point `--config` at a directory and the chain is `configuration file /tmp`, `Failed to read configuration file`, `Is a directory (os error 21)`: the same three layers, and the layer `belt` owns is the one that names the file.

Now `src/config.rs` can stop using `Box<dyn Error>`. Delete `describe` and the `use std::error::Error;` and `use confy::ConfyError;` lines, add `use crate::error::BeltError;`, and change the four signatures and their bodies:

```rust
    /// Where `belt` looks when no `--config` or `BELT_CONFIG` is given.
    pub fn default_path() -> Result<PathBuf, BeltError> {
        confy::get_configuration_file_path(APP_NAME, CONFIG_NAME).map_err(BeltError::ConfigPath)
    }

    /// Read `path`. A missing file means defaults; nothing is written to disk.
    pub fn load(path: &Path) -> Result<Self, BeltError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let config: Self = confy::load_path(path).map_err(|source| BeltError::Config {
            path: path.to_path_buf(),
            source,
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Write `self` to `path` as TOML, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> Result<(), BeltError> {
        confy::store_path(path, self).map_err(|source| BeltError::Config {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Reject values that parse fine but would break a command later.
    fn validate(&self) -> Result<(), BeltError> {
        let words = self.tag.words;
        if words == 0 || words > MAX_TAG_WORDS {
            return Err(BeltError::WordsOutOfRange { words });
        }
        Ok(())
    }
```

`map_err(BeltError::ConfigPath)` with no closure works because a tuple variant _is_ a function from its field type to the enum; `BeltError::Config` has named fields, so it needs the closure. In `src/main.rs`, the `Init` arm's `format!(...).into()` becomes `return Err(BeltError::ConfigExists { path: config_path });`, and the `Tag` arm's `tag_for_reader(...)?` becomes `tag_for_reader(...).map_err(BeltError::Input)?`. Here is what happens if you leave that last `?` bare:

```text
error[E0277]: `?` couldn't convert the error to `BeltError`
   |
80 | fn run() -> Result<(), BeltError> {
   |             --------------------- expected `BeltError` because of this
...
94 |                 tag::tag_for_reader(&mut input, config.tag.words)?;
   |                 -------------------------------------------------^ the trait `From<std::io::Error>` is not implemented for `BeltError`
```

`?` converts the error it propagates by calling [`From::from`](https://doc.rust-lang.org/std/convert/trait.From.html), which is how `Box<dyn Error>` swallowed everything in Parts 1 and 2: there is a blanket `From<E: Error>` for it. `BeltError` has no `From<io::Error>` on purpose. An `io::Error` arrives without a path, and three different variants wrap one, so the conversion has to be chosen at the call site, where the context is. Every `?` in the program is either on a `Result<_, BeltError>` already or has a `map_err` in front of it, and the compiler enforces that.

The test is the payoff. In `src/config.rs`, `too_many_words_is_rejected` changes its last assertion from a substring search to a pattern:

```rust
        let error = Config::load(&path).unwrap_err();
        assert!(matches!(error, BeltError::WordsOutOfRange { words: 65 }));
```

[`matches!`](https://doc.rust-lang.org/std/macro.matches.html) is `match` as a boolean: true when the value fits the pattern. Reword the message tomorrow and this test still passes; return the wrong variant and it fails, printing the variant it got, which is what `#[derive(Debug)]` was for.

> [!DESIGN-NOTE]
> **Why not `thiserror`?**
>
> [`thiserror`](https://docs.rs/thiserror/latest/thiserror/) is the crate most Rust projects use for exactly this file. You write the enum, put `#[error("configuration file {path}")]` above each variant and `#[source]` on the wrapped field, and a derive macro writes the `Display` and `Error` implementations you wrote by hand above, so both impl blocks disappear. `confy` uses it; the `Bad TOML data` line in the error chain earlier in this section comes from a `thiserror` attribute.
>
> This part writes the two impls out because Part 2 promised a trait implementation larger than `impl Default for TagConfig`, and because the derive is easier to read once you have seen what it generates. That is a `match` per trait, one arm per variant, and the or-pattern typing rule that decides how the `source` arms group. Switching to the derive is a mechanical change and one of the exercises. What does not change either way is the enum itself, and that is the part that matters.

## Random tags, finally

Part 1 left `belt tag` with no argument returning an error, because blocking on a terminal that is waiting for input nobody will type is the worse failure. The replacement goes in `src/tag.rs`. Add the crate:

```bash
cargo add rand
```

`Cargo.toml` gains `rand = "0.10.3"`, and `Cargo.lock` gains `rand_core` and a second `getrandom`. Then, in `src/tag.rs` under `tag_for_reader`, add one function:

```rust
/// A tag of `word_count` Bytewords drawn from the operating system's randomness.
pub fn random_tag(word_count: usize) -> String {
    let mut bytes = vec![0u8; word_count];
    rand::fill(&mut bytes[..]);
    words_for_bytes(&bytes)
}
```

[`rand::fill`](https://docs.rs/rand/0.10.3/rand/fn.fill.html) overwrites a slice with random bytes from the thread-local generator, [`ThreadRng`](https://docs.rs/rand/0.10.3/rand/rngs/struct.ThreadRng.html), which is a ChaCha stream cipher seeded from the operating system and reseeded as it goes. There is no hashing step, and none is needed: `words_for_bytes` from Part 1 maps one byte to one word, and uniformly random bytes give uniformly random words. The `vec!` instead of an array is because `word_count` comes from the config file at run time, and a stack array needs its length at compile time.

In `src/main.rs`, the `Tag { input: None }` arm loses its `TODO` and its error:

```rust
        Command::Tag { input: None } => {
            let config = Config::load(&config_path)?;
            println!("{}", tag::random_tag(config.tag.words));
        }
```

`cargo run --quiet -- tag` prints four words such as `roof-horn-very-road`, and a different four next time. A `words = 64` file prints sixty-four of them, which is what the config allows and the test for `words = 100` still prevents going past. One test, in `src/tag.rs`:

```rust
    #[test]
    fn random_tags_have_the_right_length_and_differ() {
        let first = random_tag(6);
        let second = random_tag(6);
        assert_eq!(first.split('-').count(), 6);
        assert_ne!(first, second);
    }
```

The second assertion is probabilistic, and the number is worth knowing: two six-byte tags collide with probability one in 2^48, about one in 281 trillion. If that test ever fails, rerun it; a second failure is the one worth investigating.

## Two crates called `toml`, and why this part uses the other one

The `toml` crate that `confy` pulled in during Part 2 does two things: parse text into a struct, and serialise a struct into text. Everything that is not a field of the struct, meaning comments, blank lines, the spaces around `=`, the order you wrote the keys in, is gone after the parse. For `belt config init`, which creates the file, that is fine. For a file the user has already annotated, it is destruction.

[`toml_edit`](https://docs.rs/toml_edit/0.25.17/toml_edit/) lives in the [same repository](https://github.com/toml-rs/toml) and parses the same files into a different thing: a **document**, which the crate describes as preserving _"comments, spaces and relative order of items"_. You index into it, assign a value, and call `to_string`, and the bytes you did not touch come back as they went in. The crate documents [two things it does not preserve](https://docs.rs/toml_edit/0.25.17/toml_edit/#limitations): the order of dotted keys, and a missing trailing newline. Neither matters here.

Add it, and add `etcetera` explicitly because this part needs a directory that `confy` does not expose:

```bash
cargo add toml_edit etcetera@0.10.0
```

`Cargo.toml` gains `toml_edit = "0.25.17"` and `etcetera = "0.10.0"`. The `@0.10.0` is deliberate: `confy` depends on `etcetera 0.10.0`, and asking for the current 0.11.0 would put two copies of the crate in `Cargo.lock` for no benefit. `Cargo.lock` grows by `toml_edit`, a second `toml_datetime`, 1.1.2 next to the 0.7.5 that `toml 0.9` uses, and a second `winnow`, 1.0.4 next to 0.7.15. That is the two-versions situation from the first Part 2 exercise, and it builds fine: Cargo allows two versions of one crate in a build when they are not semver-compatible, and `toml 0.9` and `toml_edit 0.25` were released against different `winnow` majors. With both crates present, `src/error.rs` compiles.

> [!ASIDE]
> `toml_edit` is what `cargo add` itself uses to write your `Cargo.toml`. The command you ran a moment ago inserted two lines into a file full of your own formatting and left the rest alone; that was `toml_edit` at work.

## `tools.node = "22"` and not one byte more

Create `src/mise.rs` with the module header and the function that does the edit. The function takes text and returns text, so it needs no file and no chezmoi to test; the `path` parameter exists only so that an error can say which file it is about:

```rust
//! `belt mise`: edit mise's global `config.toml`, directly or through chezmoi.

use std::ffi::OsStr;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use etcetera::BaseStrategy;
use toml_edit::{DocumentMut, table, value};

use crate::config::MiseConfig;
use crate::error::BeltError;

/// Set `tools.<tool>` in a TOML document, keeping every other byte as it was.
/// `path` is only used to say which file an error is about.
pub fn set_tool_in_toml(
    path: &Path,
    text: &str,
    tool: &str,
    version: &str,
) -> Result<String, BeltError> {
    let mut document: DocumentMut = text.parse().map_err(|source| BeltError::Toml {
        path: path.to_path_buf(),
        source,
    })?;
    let tools = document
        .entry("tools")
        .or_insert(table())
        .as_table_like_mut()
        .ok_or(BeltError::Shape {
            key: "tools".to_string(),
            expected: "a table",
        })?;
    tools.insert(tool, value(version));
    Ok(document.to_string())
}
```

Three names from `toml_edit` carry this. [`DocumentMut`](https://docs.rs/toml_edit/0.25.17/toml_edit/struct.DocumentMut.html) is the parsed, editable document; `text.parse()` works because the type implements `FromStr`, the same trait that lets `"42".parse::<u32>()` work. A document behaves as its root table, so [`entry("tools")`](https://docs.rs/toml_edit/0.25.17/toml_edit/struct.Table.html#method.entry) looks up the key and `or_insert(table())` creates an empty `[tools]` when there is none, in the same shape as `HashMap::entry`. [`as_table_like_mut`](https://docs.rs/toml_edit/0.25.17/toml_edit/enum.Item.html#method.as_table_like_mut) is the reason the function handles both ways of writing a table in TOML: a `[tools]` header and an inline `tools = { node = "20" }` are different types in the crate, `Table` and `InlineTable`, and `TableLike` is the trait they share. When `tools` exists and is neither, `ok_or` turns the `None` into `BeltError::Shape`, the first of the new variants to be used.

Now the test that shows this version is wrong. Append to the bottom of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// A stand-in path for the tests; it only ever appears in error text.
    fn mise_toml() -> PathBuf {
        PathBuf::from("mise.toml")
    }

    #[test]
    fn toml_edit_keeps_comments_and_replaces_the_value() {
        let before = "# my tools\n[tools]\nnode   = \"20\" # LTS\npython = \"3.12\"\n";
        let after = set_tool_in_toml(&mise_toml(), before, "node", "22").unwrap();
        assert_eq!(
            after,
            "# my tools\n[tools]\nnode   = \"22\" # LTS\npython = \"3.12\"\n"
        );
    }
}
```

`cargo test` reports:

```text
assertion `left == right` failed
  left: "# my tools\n[tools]\nnode = \"22\"\npython = \"3.12\"\n"
 right: "# my tools\n[tools]\nnode   = \"22\" # LTS\npython = \"3.12\"\n"
```

The comment at the top survived and the `python` line survived, so the document did its job. The `node` line lost its three spaces and its `# LTS`. The reason is where `toml_edit` keeps that formatting. Every key and every value carries a [`Decor`](https://docs.rs/toml_edit/0.25.17/toml_edit/struct.Decor.html), the whitespace and comments before and after it, and `insert` builds a new key and a new value with default decor and drops the old pair. The fix is to change the value that is already there instead of replacing it. Replace the `tools.insert(tool, value(version));` line with a `match` on what the key holds:

```rust
    match tools.get_mut(tool) {
        // `node = { version = "20", postinstall = "..." }`: keep the options.
        Some(item) if item.is_table_like() => {
            let options = item.as_table_like_mut().expect("checked above");
            options.insert("version", value(version));
        }
        // `node   = "20" # LTS`: keep the spacing and the comment.
        Some(item) => {
            let existing = item.as_value_mut().ok_or(BeltError::Shape {
                key: format!("tools.{tool}"),
                expected: "a value",
            })?;
            let decor = existing.decor().clone();
            *existing = version.into();
            *existing.decor_mut() = decor;
        }
        None => {
            tools.insert(tool, value(version));
        }
    }
```

The middle arm is the one the test was about: take a copy of the [`decor`](https://docs.rs/toml_edit/0.25.17/toml_edit/enum.Value.html#method.decor), overwrite the value in place with `version.into()`, which turns a `&str` into a `toml_edit::Value` string, and put the decor back. The key is never touched, so `node   =` keeps its spaces. The first arm exists because mise's `[tools]` entries are not always strings. mise accepts [tool options](https://mise.jdx.dev/dev-tools/#tool-options) written as `node = { version = "20", postinstall = "corepack enable" }`, and a version bump must not delete the `postinstall`. `is_table_like` and then `as_table_like_mut` with `expect` is the `if let` you cannot write in a match guard; the `expect` message says what was checked. The third arm is the original line, now only reached when the key is new, where default decor is the right decor.

Now the input that almost fits. `tools = "nope"` is valid TOML with `tools` as a string, so `as_table_like_mut` returns `None` and the function fails with `` `tools` is not a table``. An array of tables, `[[tools.node]]`, makes `tools.node` an item that is neither table-like nor a value, and the second arm's `ok_or` reports `` `tools.node` is not a value``. A version written as an array, `node = ["20", "22"]`, is a value, so the second arm replaces it with the string `"22"` and keeps its decor. mise [installs every version in a list](https://mise.jdx.dev/dev-tools/versions.html#multiple-versions), so `belt` has turned two installs into one. This loss is a known limit, and `belt` says nothing about it. A file that is not TOML at all fails in `parse` with `toml_edit`'s own diagnostic as the cause of `BeltError::Toml`, and the template section shows what that looks like.

Three more tests belong next to the first one. Add them inside `mod tests`:

```rust
    #[test]
    fn toml_edit_creates_the_tools_table_when_missing() {
        let after = set_tool_in_toml(&mise_toml(), "[env]\nFOO = \"bar\"\n", "node", "22").unwrap();
        assert_eq!(after, "[env]\nFOO = \"bar\"\n\n[tools]\nnode = \"22\"\n");
    }

    #[test]
    fn toml_edit_keeps_tool_options() {
        let before = "[tools]\nnode = { version = \"20\", postinstall = \"corepack enable\" }\n";
        let after = set_tool_in_toml(&mise_toml(), before, "node", "22").unwrap();
        assert_eq!(
            after,
            "[tools]\nnode = { version = \"22\", postinstall = \"corepack enable\" }\n"
        );
    }
```

The first of these records a choice `toml_edit` made for you: `or_insert(table())` appends the new table at the end of the document, header and all, after a blank line. The third test is yours: `toml_edit_rejects_a_tools_value_that_is_not_a_table`. Pass `"tools = \"nope\"\n"`, keep the error with `unwrap_err` as the Part 2 test did, and assert with `matches!` that it is `BeltError::Shape { key, expected: "a table" }` with `key == "tools"`; a guard, `if key == "tools"`, goes inside the pattern the same way it does in a `match` arm. Then move on to the file.

## Where mise keeps the file, and the direct edit

mise reads its global config from [`~/.config/mise`](https://mise.jdx.dev/configuration.html#global-config), the directory it calls `MISE_CONFIG_DIR`, in a file named `config.toml`. That is the XDG config directory plus `mise`, where XDG is the freedesktop.org base-directory layout from Part 2, and the XDG part is why `etcetera` is a direct dependency. `confy` only hands out paths under `~/.config/belt`; for `~/.config/mise`, `belt` needs the parent directory. Add these two functions above `set_tool_in_toml`:

```rust
/// Where mise reads its global configuration: `~/.config/mise/config.toml`.
pub fn global_config_path() -> Result<PathBuf, BeltError> {
    Ok(etcetera::choose_base_strategy()
        .map_err(BeltError::NoHome)?
        .config_dir()
        .join("mise/config.toml"))
}

/// Split `node@22` into `("node", "22")`.
pub fn parse_tool_spec(spec: &str) -> Result<(&str, &str), BeltError> {
    match spec.split_once('@') {
        Some((tool, version)) if !tool.is_empty() && !version.is_empty() => Ok((tool, version)),
        _ => Err(BeltError::ToolSpec {
            spec: spec.to_string(),
        }),
    }
}
```

[`choose_base_strategy`](https://docs.rs/etcetera/0.10.0/etcetera/base_strategy/fn.choose_base_strategy.html) is the directory-level cousin of the `choose_app_strategy` that `confy` calls; on Linux and macOS it returns the XDG strategy, whose `config_dir` is `$XDG_CONFIG_HOME` or `~/.config`, the same rule Part 2 traced through the spec. The `use etcetera::BaseStrategy;` at the top of the file is what makes `.config_dir()` callable: it is a trait method, and Rust only sees trait methods when the trait is in scope.

`parse_tool_spec` returns two borrowed halves of its input, `(&str, &str)`, and the compiler ties their lifetime to `spec` without being told. The guard rejects `node`, `node@` and `@22`; `node@22@x` splits at the first `@` and yields the version `22@x`, which mise will reject later with its own message. `{spec:?}` in the `Display` arm prints the input with quotes, so an empty-looking argument is visible in the error.

Now the command itself, in two halves. The direct half is this section; the chezmoi half is the next one. Add below `parse_tool_spec`:

```rust
/// Set `tools.<tool> = "<version>"` in mise's global config and make it live.
pub fn add_tool(config: &MiseConfig, tool: &str, version: &str) -> Result<(), BeltError> {
    let target = global_config_path()?;
    if config.chezmoi {
        return add_tool_via_chezmoi(&target, tool, version);
    }
    let before = match fs::read_to_string(&target) {
        Ok(text) => text,
        Err(err) if err.kind() == ErrorKind::NotFound => String::new(),
        Err(source) => return Err(file_error(&target, source)),
    };
    let after = set_tool_in_toml(&target, &before, tool, version)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| file_error(parent, source))?;
    }
    fs::write(&target, after).map_err(|source| file_error(&target, source))?;
    println!("wrote {}", target.display());
    Ok(())
}

/// Pair an I/O error with the path it happened on.
fn file_error(path: &Path, source: std::io::Error) -> BeltError {
    BeltError::File {
        path: path.to_path_buf(),
        source,
    }
}
```

A missing file is an empty document, so the first `belt mise add` on a fresh machine creates `~/.config/mise/config.toml` containing `[tools]` and one key. Every other read or write error becomes `BeltError::File` through `file_error`, which exists because the same five-line closure would otherwise appear four times in this module. This is the `map_err` discipline from the error section doing its job: no `io::Error` leaves this function without the path it happened on.

`MiseConfig` is yours to write in `src/config.rs`. It has `DecentConfig` as its template: one field, `pub chezmoi: bool`, the same four derives plus `Default`, the same `#[serde(default, deny_unknown_fields)]`, and a doc comment that says what the switch does. Add `pub mise: MiseConfig,` as the third field of `Config`. Because `bool` defaults to `false`, a `config.toml` written by Part 2's `belt config init` keeps working unchanged, and `belt config init` from now on writes an empty `[mise]` header the way it writes `[decent]`.

The command-line wiring, which is clap's job, goes in `src/main.rs`: `mod mise;` after `mod error;`, a `Mise` variant on `Command` with a `MiseAction` subcommand enum, in the shape `Config` and `ConfigAction` already have:

```rust
    /// Edit mise's global configuration.
    Mise {
        #[command(subcommand)]
        action: MiseAction,
    },
}

#[derive(Subcommand)]
enum MiseAction {
    /// Set a tool version in `[tools]`, for example `node@22`.
    Add {
        #[arg(value_name = "TOOL@VERSION")]
        spec: String,
    },
}
```

and one arm at the end of the `match` in `run`:

```rust
        Command::Mise {
            action: MiseAction::Add { spec },
        } => {
            let config = Config::load(&config_path)?;
            let (tool, version) = mise::parse_tool_spec(&spec)?;
            mise::add_tool(&config.mise, tool, version)?;
        }
```

Every `?` in that arm is on a `Result<_, BeltError>`, so none of them needs a `map_err`. `value_name = "TOOL@VERSION"` is cosmetic and worth the line: `belt mise add --help` now prints `Usage: belt mise add [OPTIONS] <TOOL@VERSION>` instead of `<SPEC>`. At this point the build fails with `cannot find function 'add_tool_via_chezmoi'`, which is the next section.

## chezmoi's idea of the file

chezmoi keeps a **source state**, a directory of files under `~/.local/share/chezmoi` by default, and computes a **target state** from it, the files it writes into your home directory. Three facts about the source state make `belt`'s job tractable, and all three are in the [source state attributes](https://www.chezmoi.io/reference/source-state-attributes/) reference:

1. The target `~/.config/mise/config.toml` is stored under a name like `dot_config/mise/config.toml`, and chezmoi spells the attributes that change how it handles the file into that filename as prefixes and suffixes. `encrypted_` at the front means the bytes on disk are ciphertext. `.tmpl` at the end means the plaintext is a template.
2. The order is fixed. For a regular file the prefixes come in the order `encrypted_`, `private_`, `readonly_`, `empty_`, `executable_`, `dot_`, so `encrypted_` is always first when it is present at all.
3. Encrypted files additionally end in `.age` or `.asc`, which chezmoi strips before it reads the other attributes. An encrypted template is therefore `encrypted_config.toml.tmpl.age`: template suffix first, encryption suffix last. The Checkpoint prints exactly that name.

The commands `belt` needs are four, each one documented as doing one thing. [`source-path`](https://www.chezmoi.io/reference/commands/source-path/) prints the source file for a target, or `not managed` on stderr with exit status 1 when chezmoi does not know the file. [`decrypt`](https://www.chezmoi.io/reference/commands/decrypt/) and [`encrypt`](https://www.chezmoi.io/reference/commands/encrypt/) take a file, or standard input when given none, and write the result to standard output. [`apply`](https://www.chezmoi.io/reference/commands/apply/) accepts a single target path. Put together:

```mermaid
flowchart TD
  Target["~/.config/mise/config.toml"] --> SP["chezmoi source-path"]
  SP --> Name{"Filename says?"}
  Name -- "encrypted_" --> Dec["chezmoi decrypt SOURCE"]
  Name -- "plain" --> Read["fs::read_to_string"]
  Dec --> Kind{".tmpl?"}
  Read --> Kind
  Kind -- no --> Toml["set_tool_in_toml"]
  Kind -- yes --> Text["set_tool_in_text"]
  Toml --> Enc{"encrypted_?"}
  Text --> Enc
  Enc -- yes --> E["chezmoi encrypt, plaintext on stdin"]
  Enc -- no --> W["fs::write SOURCE"]
  E --> W
  W --> Apply["chezmoi apply TARGET"]
  Apply --> Target
```

The code follows the diagram box for box. Add it below `add_tool`:

```rust
/// Edit the file chezmoi owns for `target`, then `chezmoi apply` it.
fn add_tool_via_chezmoi(target: &Path, tool: &str, version: &str) -> Result<(), BeltError> {
    let source_path = chezmoi(&[OsStr::new("source-path"), target.as_os_str()], b"")?;
    let source = PathBuf::from(source_path.trim_end());
    let attributes = SourceAttributes::of(&source);

    let before = if attributes.encrypted {
        chezmoi(&[OsStr::new("decrypt"), source.as_os_str()], b"")?
    } else {
        fs::read_to_string(&source).map_err(|err| file_error(&source, err))?
    };
    let after = if attributes.template {
        set_tool_in_text(&before, tool, version)
    } else {
        set_tool_in_toml(&source, &before, tool, version)?
    };
    let bytes = if attributes.encrypted {
        chezmoi(&[OsStr::new("encrypt")], after.as_bytes())?
    } else {
        after
    };
    fs::write(&source, bytes).map_err(|err| file_error(&source, err))?;
    println!("wrote {}", source.display());

    let status = Command::new("chezmoi")
        .arg("apply")
        .arg(target)
        .status()
        .map_err(BeltError::ChezmoiMissing)?;
    if !status.success() {
        return Err(BeltError::ChezmoiFailed {
            command: "apply".to_string(),
            status,
            stderr: String::new(),
        });
    }
    Ok(())
}

/// What chezmoi's source file name says about the file.
struct SourceAttributes {
    /// `encrypted_` prefix: the bytes on disk are ciphertext.
    encrypted: bool,
    /// `.tmpl` suffix: the plaintext is a Go template, not plain TOML.
    template: bool,
}

impl SourceAttributes {
    fn of(source: &Path) -> Self {
        let name = source
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        // chezmoi appends .age or .asc after every other attribute.
        let name = name
            .strip_suffix(".age")
            .or_else(|| name.strip_suffix(".asc"))
            .unwrap_or(name);
        Self {
            encrypted: name.starts_with("encrypted_"),
            template: name.ends_with(".tmpl"),
        }
    }
}
```

`encrypt` is fed the plaintext on standard input instead of through a temporary file, so the decrypted content never touches the disk in this process; `chezmoi edit` [decrypts to a private temporary directory](https://www.chezmoi.io/reference/commands/edit/) because an editor needs a file, and `belt` does not. `source.as_os_str()` instead of a `&str` is why the helper below takes `&[&OsStr]`: a path on Unix is bytes, not necessarily UTF-8, and `Command` accepts it as it is. The ciphertext that `encrypt` returns is ASCII armour, so the helper can hand back a `String` for every command and `fs::write` takes it as bytes.

The code runs `apply` differently from the other three, and on purpose. The helper that follows captures standard output and error so that `belt` can report `chezmoi source-path: exit status: 1: chezmoi: /Users/you/.config/mise/config.toml: not managed` in one line, through the `ChezmoiFailed` variant. `apply` instead inherits your terminal, because when the target was [modified since chezmoi last wrote it](https://www.chezmoi.io/reference/commands/apply/), chezmoi asks `has changed since chezmoi last wrote it?` and waits for a key. With its input redirected it cannot, and fails with `could not open a new TTY`. Inheriting the terminal gives the user the prompt chezmoi meant them to see, and leaves `stderr` empty in the error, because chezmoi already printed it.

> [!HEADS-UP]
> That prompt is exactly the `mise use --global` situation from the opening. If you ran it before switching to `belt`, the first `belt mise add` will stop at chezmoi's question. Answer it once; after that, `belt` only ever writes the source, and the target stays in step.

The helper:

```rust
/// Run `chezmoi` with `args`, feed it `stdin`, and return its stdout as text.
fn chezmoi(args: &[&OsStr], stdin: &[u8]) -> Result<String, BeltError> {
    let command = args[0].to_string_lossy().into_owned();
    let mut child = Command::new("chezmoi")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(BeltError::ChezmoiMissing)?;
    child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(stdin)
        .map_err(BeltError::ChezmoiMissing)?;
    let output = child
        .wait_with_output()
        .map_err(BeltError::ChezmoiMissing)?;
    if !output.status.success() {
        return Err(BeltError::ChezmoiFailed {
            command,
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    String::from_utf8(output.stdout).map_err(|source| BeltError::ChezmoiOutput { command, source })
}
```

This is the function the hook promised, with its three failures: `ChezmoiMissing` when the binary cannot be started, `ChezmoiFailed` when it exits non-zero, `ChezmoiOutput` when it prints something that is not text. The first of those reads, in full, `error: could not run chezmoi` and then `caused by: No such file or directory (os error 2)`, which is what a user without chezmoi installed sees. [`Stdio::piped()`](https://doc.rust-lang.org/std/process/struct.Stdio.html#method.piped) on all three streams means `belt` owns them. `child.stdin.take()` moves the write end out of the `Child` so that it is dropped, and so closed, as soon as `write_all` returns; without that close, `chezmoi encrypt` would wait for more input forever. [`wait_with_output`](https://doc.rust-lang.org/std/process/struct.Child.html#method.wait_with_output) then drains the other two streams and waits for exit.

> [!DESIGN-NOTE]
> **Why not `chezmoi edit` with `belt` as the editor?**
>
> `chezmoi edit` already does the decrypt, edit, re-encrypt dance, and it chooses its editor from [`edit.command` in the chezmoi config, then `$VISUAL`, then `$EDITOR`](https://www.chezmoi.io/reference/configuration-file/variables/). Setting `EDITOR=belt mise edit-file` before calling `chezmoi edit ~/.config/mise/config.toml` would let chezmoi do everything in this section.
>
> Three things argue against it. A user who has set `edit.command` wins over the environment, so the trick silently opens their editor instead. `belt` would need a hidden subcommand whose only purpose is to be called back, and a path to it that survives a shell-style split. And `chezmoi edit` applies only when told to with `--apply`, where `mise use` makes the new version live immediately; matching mise is the behaviour a `mise` subcommand should have. Three `Command` calls and one `status()` are less machinery than that.
>
> There is one thing `chezmoi edit` and `belt` both leave undone: the source directory is usually a git repository, and nothing here commits. `chezmoi git -- commit -am 'node 22'` after the fact, or `git.autoCommit = true` in the chezmoi config, are the two ways to close that gap, and one of the exercises is the first of them.

## A template is not TOML

Here is what happens when the file chezmoi hands back contains template code and `belt` parses it as TOML anyway. The top line is `BeltError::Toml`'s; the cause is `toml_edit`'s:

```text
error: could not edit /tmp/belt-part3/home/.config/mise/config.toml
  caused by: TOML parse error at line 3, column 1
  |
3 | {{- if eq .chezmoi.os "darwin" }}
  | ^
invalid key-value pair, expected key
```

A line that starts with `{{` is not a key, and no TOML parser can be talked into thinking otherwise. Rendering the template first is not an answer either, because the thing to edit is the template, not its output; rendering it would produce the same file `mise use` wrote, with the branches for the other operating systems gone. The only edit that respects a template it does not understand is a textual one, and TOML's own rules make a textual edit safe enough as long as it stays inside the right section. Appending `[tools]` to the end of a file that already has one is not an option: the specification says [a table cannot be defined more than once](https://toml.io/en/v1.0.0#table), and mise would refuse the file. So the editor finds the existing header, and only appends a new one when there is none:

```rust
/// A template is not TOML to a parser, so edit it line by line: replace the
/// key inside `[tools]`, insert it under the header, or append the table.
///
/// shortcut: edits the first `[tools]` header only and does not see dotted
/// keys or inline tables; upgrade to a template-aware pass if a file needs it.
pub fn set_tool_in_text(text: &str, tool: &str, version: &str) -> String {
    let assignment = format!("{tool} = \"{version}\"");
    let mut lines: Vec<&str> = text.lines().collect();

    let Some(header) = lines.iter().position(|line| line.trim() == "[tools]") else {
        let mut edited = text.to_string();
        if !edited.is_empty() && !edited.ends_with('\n') {
            edited.push('\n');
        }
        edited.push_str(&format!("\n[tools]\n{assignment}\n"));
        return edited;
    };

    let section_end = lines[header + 1..]
        .iter()
        .position(|line| line.trim_start().starts_with('['))
        .map_or(lines.len(), |offset| header + 1 + offset);
    let existing = lines[header + 1..section_end]
        .iter()
        .position(|line| {
            line.split_once('=')
                .is_some_and(|(key, _)| key.trim() == tool)
        })
        .map(|offset| header + 1 + offset);

    match existing {
        Some(index) => lines[index] = &assignment,
        None => lines.insert(header + 1, &assignment),
    }
    let mut edited = lines.join("\n");
    edited.push('\n');
    edited
}
```

`let Some(header) = ... else { ... }` is a **let-else**: bind the value if the pattern matches, otherwise run the block, which must leave the function. It replaces a `match` whose other arm would have been four lines of early return. The section runs from the line after the header to the next line that starts with `[`, which is the next header; `section_end` falls back to the end of the file when there is no next header. Inside the section, a line belongs to the tool when the text before its first `=` trims to the tool name, so `# node = "20"` does not match and `node = "20"` does. The editor replaces a match in place; no match means the assignment goes directly under the header, where a reader sees it first. `text.lines()` discards the final newline, which is why the function puts one back. This function returns a plain `String`, not a `Result`: a line edit on text has no way to fail.

The doc comment states the limits, and the word `shortcut` in it is deliberate so that a search finds them later. If `[tools]` appears twice, once per template branch, the editor edits only the first. If the header is inside a `{{- if }}` block, so is the new key. This editor does not parse values, so it replaces a tool written as an inline table on one line with a string, options and all. Dotted keys at the top of the file, `tools.node = "20"`, are invisible to it. The chezmoi-managed mise config this part was written against has `[tools]` once, outside any conditional, with `{{- if }}` blocks nested below it, and that is the shape the tests use:

```rust
    const TEMPLATE: &str = "[settings]\nexperimental = true\n\n[tools]\nnode = \"20\"\n{{- if eq .chezmoi.os \"darwin\" }}\nxcodes = \"latest\"\n{{- end }}\n\n[env]\nFOO = \"bar\"\n";

    #[test]
    fn text_edit_replaces_inside_the_tools_section_only() {
        let after = set_tool_in_text(TEMPLATE, "node", "22");
        assert_eq!(after, TEMPLATE.replace("node = \"20\"", "node = \"22\""));
    }

    #[test]
    fn text_edit_inserts_under_the_header() {
        let after = set_tool_in_text(TEMPLATE, "python", "3.12");
        assert_eq!(
            after,
            TEMPLATE.replace("[tools]\n", "[tools]\npython = \"3.12\"\n")
        );
    }

    #[test]
    fn text_edit_appends_the_table_when_missing() {
        let after = set_tool_in_text("[env]\nFOO = \"bar\"", "node", "22");
        assert_eq!(after, "[env]\nFOO = \"bar\"\n\n[tools]\nnode = \"22\"\n");
    }
```

The expected values are written as edits to `TEMPLATE` instead of as second literals, so that the test says what changed and nothing else. The third test feeds a file with no trailing newline, the one case the first branch has to repair before appending. One more test is yours, and it is the smallest in the file: `tool_spec_needs_both_halves`, which asserts that `parse_tool_spec("node@22")` unwraps to `("node", "22")`, then loops over `["node", "node@", "@22"]` and asserts with `matches!` that each one is `Err(BeltError::ToolSpec { spec })` with `spec` equal to the input. Write it, then run the Checkpoint.

One more thing about the choice between the two editors: `belt` makes it from the filename, not by trying `toml_edit` first and falling back on failure. A plain file with a typo in it would fail the parser, get the line editor, and have a key appended to a broken file before `chezmoi apply` ran. The filename is chezmoi's statement of intent; the parser's verdict is not.

## Checkpoint

> [!PREDICT]
> After the last `belt mise add` below, two files hold the result: the source `encrypted_config.toml.age` and the target `config.toml`. Which of them contains the bytes `python = "3.12"`, and what does the other one start with?

**Run this to verify your work so far:**

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo build --quiet
BELT=$PWD/target/debug/belt
export HOME=/tmp/belt-part3/home
mkdir -p $HOME/.config/chezmoi $HOME/.config/mise $HOME/.config/belt
$BELT tag
printf 'hello world' | $BELT tag -
printf '[tag]\nwords = 100\n' > $HOME/.config/belt/config.toml
$BELT tag
printf '# pinned by hand\n[tools]\nnode   = "20" # LTS\n' > $HOME/.config/mise/config.toml
printf '[mise]\nchezmoi = false\n' > $HOME/.config/belt/config.toml
$BELT mise add node@22
cat $HOME/.config/mise/config.toml
chezmoi age-keygen --output=$HOME/key.txt
printf 'encryption = "age"\n[age]\n    identity = "%s/key.txt"\n    recipient = "%s"\n' "$HOME" "$(grep -o 'age1[a-z0-9]*' $HOME/key.txt)" > $HOME/.config/chezmoi/chezmoi.toml
chezmoi add --encrypt $HOME/.config/mise/config.toml
chezmoi source-path $HOME/.config/mise/config.toml
printf '[mise]\nchezmoi = true\n' > $HOME/.config/belt/config.toml
$BELT mise add python@3.12
chezmoi cat $HOME/.config/mise/config.toml
chezmoi status
head -c 40 $(chezmoi source-path $HOME/.config/mise/config.toml); echo
```

The `export HOME` line is what makes this safe to run on your real machine: chezmoi, `etcetera` and `confy` all derive their directories from `HOME`, so everything below it lands in `/tmp/belt-part3/home` and your own dotfiles are never read. Build first and keep using `$BELT` because once `HOME` has moved, `cargo` would look for its toolchain under the new one and fail. `chezmoi age-keygen --output` writes the identity to the file and prints nothing; the recipient is on the `# public key:` line inside it, which the `grep` extracts.

Expected output, trimmed of the `Compiling` lines, with the first line after the tests being four words of your own:

```text
running 15 tests
test config::tests::defaults_round_trip_through_disk ... ok
test config::tests::missing_file_means_defaults_and_no_write ... ok
test config::tests::too_many_words_is_rejected ... ok
test mise::tests::text_edit_appends_the_table_when_missing ... ok
test mise::tests::text_edit_inserts_under_the_header ... ok
test mise::tests::text_edit_replaces_inside_the_tools_section_only ... ok
test mise::tests::toml_edit_creates_the_tools_table_when_missing ... ok
test mise::tests::toml_edit_keeps_comments_and_replaces_the_value ... ok
test mise::tests::toml_edit_keeps_tool_options ... ok
test mise::tests::toml_edit_rejects_a_tools_value_that_is_not_a_table ... ok
test mise::tests::tool_spec_needs_both_halves ... ok
test tag::tests::empty_input_still_gets_a_tag ... ok
test tag::tests::hello_world_has_a_known_tag ... ok
test tag::tests::one_byte_changes_the_tag ... ok
test tag::tests::random_tags_have_the_right_length_and_differ ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
roof-horn-very-road
also-code-wave-list
error: tag.words must be 1 to 64, got 100
wrote /tmp/belt-part3/home/.config/mise/config.toml
# pinned by hand
[tools]
node   = "22" # LTS
/tmp/belt-part3/home/.local/share/chezmoi/dot_config/mise/encrypted_config.toml.age
wrote /tmp/belt-part3/home/.local/share/chezmoi/dot_config/mise/encrypted_config.toml.age
# pinned by hand
[tools]
node   = "22" # LTS
python = "3.12"
-----BEGIN AGE ENCRYPTED FILE-----
YWdlL
```

The test lines may come out in a different order. The random tag is random; `hello world` is still `also-code-wave-list`, the same as Part 1, because the hash has not changed. The `words = 100` error is one line with no `caused by`, because `WordsOutOfRange` is a leaf. The first `wrote` is the direct edit: `node   = "22" # LTS` with its spacing and comment. After `chezmoi add --encrypt`, the source filename carries both `encrypted_` and `.age`, and the second `wrote` names that file; `belt` decrypted it, appended `python` at the end of `[tools]`, re-encrypted and applied. `chezmoi cat` renders the target state and shows the new key; `chezmoi status` prints nothing, which means the target on disk matches the source. The last line is the first 40 bytes of the source: ciphertext, which answers the prediction. The plaintext exists only in the target and, briefly, in `belt`'s memory.

To see the template path as well, run `chezmoi chattr +template $HOME/.config/mise/config.toml`, then `chezmoi source-path` again: the name becomes `encrypted_config.toml.tmpl.age`, and the next `belt mise add` goes through `set_tool_in_text`. `rm -r /tmp/belt-part3` when you are done, and open a new shell instead of trying to put `HOME` back.

**Likely errors:**

- If you see `error[E0004]: non-exhaustive patterns: '&BeltError::...' not covered` pointing into `src/error.rs`, one of the two `match` blocks is missing a variant. The message names it.
- If you see `error[E0277]: '?' couldn't convert the error to 'BeltError'`, a `?` sits on a `Result` whose error type is not `BeltError`. The message says which type; wrap the call in `map_err` with the matching variant.
- If you see `error[E0004]: non-exhaustive patterns: Command::Mise { .. } not covered`, the `Mise` arm is missing from the `match` in `run`.
- If you see `error[E0432]: unresolved import 'etcetera'`, run the `cargo add` line from the `toml` section; `confy` depends on `etcetera`, but that does not make the crate visible to your code.
- If you see `error[E0599]: no method named 'config_dir' found for struct 'etcetera::base_strategy::Xdg'`, the `use etcetera::BaseStrategy;` line is missing. The method exists, but the trait that provides it is not in scope.
- If the first `toml_edit` test fails with `left: "# my tools\n[tools]\nnode = \"22\"\n...`, `set_tool_in_toml` is still calling `insert` on an existing key. Apply the three-arm `match`.
- If `belt mise add` says `error: could not run chezmoi` with `caused by: No such file or directory (os error 2)`, chezmoi is not on your `PATH`; the direct mode and the tests still work.
- If it says `error: chezmoi source-path: exit status: 1: chezmoi: /tmp/belt-part3/home/.config/mise/config.toml: not managed`, the `chezmoi add --encrypt` line was skipped, or one of the commands ran with a different `HOME` than the others.
- If `chezmoi add` fails with `age: error: missing recipients`, the `recipient` line in `chezmoi.toml` is empty; check that `grep -o 'age1[a-z0-9]*' $HOME/key.txt` prints a key.
- If `belt mise add` stops at `has changed since chezmoi last wrote it?`, the target was edited behind chezmoi's back, by hand or by `mise use`. Answer the prompt; in a script with no terminal the same situation fails with `could not open a new TTY`.

## What's next

`belt mise` has `add` and nothing else, and `BeltError` has twelve variants that all describe things going wrong on one machine. The next part starts `belt decent`, the command group from Part 1's tree that the `[decent]` table has been waiting for since Part 2 with its unused `broker` key. It is the first command that opens a network connection, and a connection brings the first error `belt` does not own and cannot name from a filename: a failure from the client library, arriving at run time, from a machine that is not yours. The question that part answers is how an error type that was designed around paths grows a variant for that without turning back into a bag of strings.

Before you try the exercises: in two sentences, explain to a sceptical colleague why `BeltError` has no `impl From<io::Error>`, even though adding one would delete every `map_err` in `src/mise.rs`. Your answer should name what the error message loses.

## Exercises

- [ ] Switch `src/error.rs` to [`thiserror`](https://docs.rs/thiserror/latest/thiserror/): `cargo add thiserror`, put `#[error("...")]` above each variant and `#[source]` on each wrapped field, and delete both hand-written impls. The tests and the Checkpoint output must not change; `cargo expand` shows you what the derive wrote in their place.
- [ ] Add `belt mise remove TOOL`. In the TOML path, `TableLike::remove` deletes the key and the decor goes with it; check whether a trailing comment on the removed line survives, and decide whether a now-empty `[tools]` header should be removed too. In the text path, delete the matching line.
- [ ] mise reads `MISE_CONFIG_DIR` and `MISE_GLOBAL_CONFIG_FILE` before it falls back to `~/.config/mise/config.toml`. Make `global_config_path` honour both, in [the order mise documents](https://mise.jdx.dev/configuration.html#global-config), and add a test that sets each variable with `std::env::set_var` inside an `unsafe` block, which is what edition 2024 requires.
- [ ] After `fs::write` in `add_tool_via_chezmoi`, run `chezmoi git -- add` and `chezmoi git -- commit -m "mise: node 22"` so the source repository records the change. Decide what `belt` must do when the source directory is not a git repository, give that outcome its own `BeltError` variant, and make the test for it the first thing you write.
- [ ] Verify before you write. In the template path, after `set_tool_in_text` and before `fs::write`, ask chezmoi to render the edited template with `chezmoi execute-template` on the new text, parse the rendering with `toml_edit`, and fail with `BeltError::Toml` if it does not parse. Measure how much slower `belt mise add` becomes.

## Sources

The error type and random tags:

1. [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html) — no required methods; `source` as "the lower-level source of this error"; the `Debug + Display` supertraits.
2. [`std::fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) — the one-method trait behind `{}`.
3. [`std::convert::From`](https://doc.rust-lang.org/std/convert/trait.From.html) — the conversion `?` performs on the error it propagates.
4. [`matches!`](https://doc.rust-lang.org/std/macro.matches.html) — pattern matching as a boolean, with optional guard.
5. [`thiserror`](https://docs.rs/thiserror/latest/thiserror/) — the derive that writes the two impls; the alternative in the design note and the first exercise.
6. [`rand::fill`](https://docs.rs/rand/0.10.3/rand/fn.fill.html) and [`rand::rngs::ThreadRng`](https://docs.rs/rand/0.10.3/rand/rngs/struct.ThreadRng.html) — filling a slice from the thread-local generator; a ChaCha-based generator seeded from the operating system and periodically reseeded.

chezmoi and mise:

7. [chezmoi: source state attributes](https://www.chezmoi.io/reference/source-state-attributes/) — the prefix and suffix tables, the fixed prefix order with `encrypted_` first, and the `.age` and `.asc` suffixes that are stripped before the others are read.
8. [chezmoi: `source-path`](https://www.chezmoi.io/reference/commands/source-path/), [`decrypt`](https://www.chezmoi.io/reference/commands/decrypt/), [`encrypt`](https://www.chezmoi.io/reference/commands/encrypt/) and [`apply`](https://www.chezmoi.io/reference/commands/apply/) — the four commands `belt` runs; `decrypt` and `encrypt` read standard input when given no file and write standard output; `apply` prompts when a target has changed since chezmoi last wrote it.
9. [chezmoi: `edit`](https://www.chezmoi.io/reference/commands/edit/) and [configuration variables](https://www.chezmoi.io/reference/configuration-file/variables/) — decrypts to a private temporary directory and re-encrypts afterwards; `--apply` is opt-in; `edit.command`, `$VISUAL`, `$EDITOR`; the alternative the design note rejects.
10. [chezmoi: `status`](https://www.chezmoi.io/reference/commands/status/) — the two-column format and what `MM` means.
11. [chezmoi: encryption](https://www.chezmoi.io/user-guide/encryption/) and [age](https://www.chezmoi.io/user-guide/encryption/age/) — ASCII-armoured storage with the `encrypted_` attribute, the `encryption = "age"` configuration, and the built-in age implementation used when no `age` binary is on `PATH`.
12. [mise: global and system config](https://mise.jdx.dev/configuration.html#global-config) — `~/.config/mise` as `MISE_CONFIG_DIR`, `config.toml`, and the fact that `mise use --global` writes to it.
13. [mise: `use`](https://mise.jdx.dev/cli/use.html) — `--global` writes the global config file, falling back to `~/.config/mise/config.toml`; [mise: multiple versions](https://mise.jdx.dev/dev-tools/versions.html#multiple-versions) — mise installs every version in a list and the first provides the command.
14. [mise: tool options](https://mise.jdx.dev/dev-tools/#tool-options) — the `{ version = "...", postinstall = "..." }` form that the first `match` arm preserves.

TOML and the crates that edit it:

15. [`toml_edit` crate docs](https://docs.rs/toml_edit/0.25.17/toml_edit/) — "comments, spaces and relative order" preserved; the two documented limitations.
16. [`toml_edit::Table::entry`](https://docs.rs/toml_edit/0.25.17/toml_edit/struct.Table.html#method.entry), [`Item::as_table_like_mut`](https://docs.rs/toml_edit/0.25.17/toml_edit/enum.Item.html#method.as_table_like_mut), [`Value::decor`](https://docs.rs/toml_edit/0.25.17/toml_edit/enum.Value.html#method.decor) and [`Decor`](https://docs.rs/toml_edit/0.25.17/toml_edit/struct.Decor.html) — the API the edit is built from, and where the whitespace and comments live.
17. [`etcetera::choose_base_strategy`](https://docs.rs/etcetera/0.10.0/etcetera/base_strategy/fn.choose_base_strategy.html) — the directory-level strategy; XDG on Linux and macOS.
18. [`std::process::Stdio::piped`](https://doc.rust-lang.org/std/process/struct.Stdio.html#method.piped) and [`Child::wait_with_output`](https://doc.rust-lang.org/std/process/struct.Child.html#method.wait_with_output) — owning a child's streams and draining them.
19. [TOML v1.0.0: Table](https://toml.io/en/v1.0.0#table) — "you cannot define a table more than once".
20. [toml-rs repository](https://github.com/toml-rs/toml) — `toml` and `toml_edit` share a repository and a parser lineage, which is why the diagnostics look alike.
