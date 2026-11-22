mod pw;

use std::io::{self, Write};
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "belt", version, about, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a password from four English BIP39 words.
    Pw {
        /// Require a password strictly longer than MIN_LENGTH.
        /// The default gives at least 17 characters; smaller values are allowed.
        #[arg(default_value_t = 16)]
        min_length: usize,
    },
}

fn run() -> io::Result<()> {
    match Cli::parse().command {
        Some(Command::Pw { min_length }) => {
            let password = pw::generate_password(min_length, &mut io::stderr().lock())?;
            writeln!(io::stdout().lock(), "{password}")
        }
        None => Cli::command().print_help(),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smaller_positional_value_replaces_the_default() {
        let cli = Cli::try_parse_from(["belt", "pw", "8"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Pw { min_length: 8 })));
    }
}
