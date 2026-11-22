use std::process::{Command, Output};
use std::str;

const RETRY_NOTICE: &str = "Password was too short; regenerating.";

fn belt(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_belt"))
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("could not run belt")
}

fn assert_password(output: &Output, min_length: usize) {
    assert!(output.status.success());
    let password = str::from_utf8(&output.stdout)
        .unwrap()
        .strip_suffix('\n')
        .expect("missing final newline");
    assert!(password.len() > min_length);
    let words: Vec<_> = password
        .split(|letter: char| letter.is_ascii_digit() || "&+.,/-=?@#^%".contains(letter))
        .collect();
    assert_eq!(words.len(), 4);
    assert!(words.iter().all(|word| !word.is_empty()
        && word.bytes().all(|letter| letter.is_ascii_alphabetic())));
}

#[test]
fn default_prints_only_one_password_longer_than_sixteen() {
    let output = belt(&["pw"]);
    assert_password(&output, 16);
    let diagnostics = str::from_utf8(&output.stderr).unwrap();
    assert!(diagnostics.lines().all(|line| line == RETRY_NOTICE));
}

// The unit tests in `src/main.rs` check that zero reaches the generator; this checks the process.
#[test]
fn zero_minimum_prints_a_password_without_diagnostics() {
    let output = belt(&["pw", "0"]);
    assert_password(&output, 0);
    assert!(output.stderr.is_empty());
}

#[test]
fn warned_request_prints_one_warning_then_only_retry_notices_to_stderr() {
    let output = belt(&["pw", "26"]);
    assert_password(&output, 26);
    let mut lines = str::from_utf8(&output.stderr).unwrap().lines();
    assert_eq!(
        lines.next(),
        Some("Warning: a minimum length of 26 significantly restricts password entropy.")
    );
    assert!(lines.all(|line| line == RETRY_NOTICE));
}

#[test]
fn refused_request_explains_failure_without_stdout() {
    let output = belt(&["pw", "32"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let diagnostics = str::from_utf8(&output.stderr).unwrap();
    assert!(diagnostics.contains("refusing to generate a password"));
    assert!(diagnostics.contains("32") && diagnostics.contains("95%"));
}

#[test]
fn malformed_or_negative_length_is_a_usage_error_without_stdout() {
    for invalid in ["not-a-number", "-1"] {
        let output = belt(&["pw", invalid]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(str::from_utf8(&output.stderr).unwrap().contains("error:"));
    }
}

#[test]
fn pw_help_explains_the_exclusive_default() {
    let output = belt(&["pw", "--help"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let help = str::from_utf8(&output.stdout).unwrap();
    assert!(help.contains("belt pw [MIN_LENGTH]"));
    assert!(help.contains("strictly longer"));
    assert!(help.contains("[default: 16]"));
}

#[test]
fn bare_belt_prints_top_level_help_successfully() {
    let output = belt(&[]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let help = str::from_utf8(&output.stdout).unwrap();
    assert!(help.contains("Usage: belt"));
    assert!(help.contains("Commands:") && help.contains("pw"));
}
