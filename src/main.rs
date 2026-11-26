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
    /// Generate a password from five English BIP39 words.
    Pw {
        /// Require a password strictly longer than MIN_LENGTH.
        /// The default gives at least 19 characters; smaller values are allowed.
        #[arg(default_value_t = pw::DEFAULT_MIN_LENGTH)]
        min_length: usize,
    },
}

fn run(
    command: Option<Command>,
    output: &mut impl Write,
    generate_password: impl FnOnce(usize) -> io::Result<String>,
) -> io::Result<()> {
    match command {
        Some(Command::Pw { min_length }) => {
            let password = generate_password(min_length)?;
            writeln!(output, "{password}")
        }
        None => Cli::command().print_help(),
    }
}

fn main() -> ExitCode {
    let command = Cli::parse().command;
    let result = run(command, &mut io::stdout().lock(), |min_length| {
        pw::generate_password(min_length, &mut io::stderr().lock())
    });
    match result {
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
    fn positional_value_including_zero_reaches_the_generator_in_place_of_the_default() {
        for (arguments, expected_minimum) in [
            (&["belt", "pw"][..], 18),
            (&["belt", "pw", "8"][..], 8),
            (&["belt", "pw", "0"][..], 0),
        ] {
            let command = Cli::try_parse_from(arguments).unwrap().command;
            let mut requested_minimum = None;
            let mut output = Vec::new();
            run(command, &mut output, |min_length| {
                requested_minimum = Some(min_length);
                Ok("generated".to_owned())
            })
            .unwrap();
            assert_eq!(
                requested_minimum,
                Some(expected_minimum),
                "arguments: {arguments:?}"
            );
            assert_eq!(output, b"generated\n");
        }
    }
}
