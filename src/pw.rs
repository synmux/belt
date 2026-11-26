use std::io::{self, Write};

use bip39::Language;
use rand::{
    CryptoRng, RngExt, SeedableRng, TryCryptoRng,
    rngs::{StdRng, SysRng},
    seq::SliceRandom,
};

pub const DEFAULT_MIN_LENGTH: usize = 18;
const WORD_COUNT: usize = 5;
const GAP_COUNT: usize = WORD_COUNT - 1;
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"&+.,/-=?@#^%";
const GAP_PATTERNS: [[&[u8]; GAP_COUNT]; 14] = [
    [DIGITS, DIGITS, DIGITS, SYMBOLS],
    [DIGITS, DIGITS, SYMBOLS, DIGITS],
    [DIGITS, DIGITS, SYMBOLS, SYMBOLS],
    [DIGITS, SYMBOLS, DIGITS, DIGITS],
    [DIGITS, SYMBOLS, DIGITS, SYMBOLS],
    [DIGITS, SYMBOLS, SYMBOLS, DIGITS],
    [DIGITS, SYMBOLS, SYMBOLS, SYMBOLS],
    [SYMBOLS, DIGITS, DIGITS, DIGITS],
    [SYMBOLS, DIGITS, DIGITS, SYMBOLS],
    [SYMBOLS, DIGITS, SYMBOLS, DIGITS],
    [SYMBOLS, DIGITS, SYMBOLS, SYMBOLS],
    [SYMBOLS, SYMBOLS, DIGITS, DIGITS],
    [SYMBOLS, SYMBOLS, DIGITS, SYMBOLS],
    [SYMBOLS, SYMBOLS, SYMBOLS, DIGITS],
];

fn generate_candidate(rng: &mut impl CryptoRng) -> String {
    let dictionary = Language::English.word_list();
    let uppercase_count = rng.random_range(1..WORD_COUNT);
    let mut uppercase_positions = [false; WORD_COUNT];
    uppercase_positions[..uppercase_count].fill(true);
    uppercase_positions.shuffle(rng);
    let gaps = GAP_PATTERNS[rng.random_range(0..GAP_PATTERNS.len())];

    let mut candidate = String::new();
    for (position, uppercase) in uppercase_positions.into_iter().enumerate() {
        let word = dictionary[rng.random_range(0..dictionary.len())];
        if uppercase {
            candidate.extend(word.chars().map(|letter| letter.to_ascii_uppercase()));
        } else {
            candidate.push_str(word);
        }
        if let Some(alphabet) = gaps.get(position) {
            candidate.push(char::from(alphabet[rng.random_range(0..alphabet.len())]));
        }
    }
    candidate
}

fn length_distribution() -> Vec<u64> {
    let dictionary = Language::English.word_list();
    let longest_word = dictionary
        .iter()
        .map(|word| word.len())
        .max()
        .expect("non-empty BIP39 dictionary");
    let mut word_counts = vec![0_u64; longest_word + 1];
    for word in dictionary {
        word_counts[word.len()] += 1;
    }

    let mut counts = vec![0; GAP_COUNT + 1];
    counts[GAP_COUNT] = 1;
    for _ in 0..WORD_COUNT {
        let mut next = vec![0; counts.len() + longest_word];
        for (length, ways) in counts.iter().enumerate() {
            for (word_length, frequency) in word_counts.iter().enumerate() {
                next[length + word_length] += ways * frequency;
            }
        }
        counts = next;
    }
    counts
}

#[derive(Debug, PartialEq, Eq)]
enum LengthPolicy {
    Allow,
    Warn,
    Refuse,
}

fn length_policy(min_length: usize) -> LengthPolicy {
    let counts = length_distribution();
    let rejected: u64 = counts.iter().take(min_length.saturating_add(1)).sum();
    let total: u64 = counts.iter().sum();
    if 100 * rejected > 95 * total {
        LengthPolicy::Refuse
    } else if 100 * rejected > 70 * total {
        LengthPolicy::Warn
    } else {
        LengthPolicy::Allow
    }
}

pub fn generate_password(min_length: usize, diagnostics: &mut impl Write) -> io::Result<String> {
    generate_seeded(min_length, diagnostics, &mut SysRng)
}

/// Seeds the generator from `entropy`. A seeding failure is returned as an error; there is no
/// fallback to a weaker source.
fn generate_seeded<Entropy: TryCryptoRng + ?Sized>(
    min_length: usize,
    diagnostics: &mut impl Write,
    entropy: &mut Entropy,
) -> io::Result<String> {
    generate_with(min_length, diagnostics, || {
        let mut rng = StdRng::try_from_rng(entropy).map_err(|error| {
            io::Error::other(format!(
                "could not seed the password generator from the operating system: {error}"
            ))
        })?;
        Ok(move || generate_candidate(&mut rng))
    })
}

fn generate_with<NextCandidate>(
    min_length: usize,
    diagnostics: &mut impl Write,
    start_generator: impl FnOnce() -> io::Result<NextCandidate>,
) -> io::Result<String>
where
    NextCandidate: FnMut() -> String,
{
    match length_policy(min_length) {
        LengthPolicy::Refuse => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "refusing to generate a password: a minimum length of {min_length} rejects more than 95% of all five-word combinations, which leaves you with terrible password entropy. It'll be more secure with fewer characters. Ironically enough."
                ),
            ));
        }
        // Diagnostics are best-effort: a broken stderr must not withhold the password.
        LengthPolicy::Warn => {
            let _ = writeln!(
                diagnostics,
                "Warning: a minimum length of {min_length} significantly restricts password entropy."
            );
        }
        LengthPolicy::Allow => {}
    }

    let mut next_candidate = start_generator()?;
    loop {
        let candidate = next_candidate();
        if candidate.len() > min_length {
            return Ok(candidate);
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::TryRng;

    use super::*;

    #[test]
    fn distribution_counts_ordered_word_tuples_including_joiners() {
        let counts = length_distribution();
        let total: u64 = counts.iter().sum();
        assert_eq!(total, 2048_u64.pow(5));
        let total = u128::from(total);
        for (minimum, basis_points) in [(31, 5696), (32, 7004), (35, 9438), (36, 9745)] {
            let rejected = u128::from(counts[..=minimum].iter().sum::<u64>());
            assert_eq!((10_000 * rejected + total / 2) / total, basis_points);
        }
    }

    #[test]
    fn default_is_the_longest_minimum_that_rejects_no_combinations() {
        let counts = length_distribution();
        assert_eq!(counts[..=DEFAULT_MIN_LENGTH].iter().sum::<u64>(), 0);
        assert!(counts[DEFAULT_MIN_LENGTH + 1] > 0);
    }

    #[test]
    fn classifies_both_entropy_thresholds() {
        for (minimum, expected) in [
            (0, LengthPolicy::Allow),
            (31, LengthPolicy::Allow),
            (32, LengthPolicy::Warn),
            (35, LengthPolicy::Warn),
            (36, LengthPolicy::Refuse),
            (40, LengthPolicy::Refuse),
            (usize::MAX, LengthPolicy::Refuse),
        ] {
            assert_eq!(length_policy(minimum), expected, "minimum: {minimum}");
        }
    }

    #[test]
    fn refuses_before_initialising_the_generator() {
        let mut diagnostics = Vec::new();
        let error = generate_with(
            usize::MAX,
            &mut diagnostics,
            || -> io::Result<fn() -> String> {
                panic!("generator initialised for a refused minimum")
            },
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("95%"));
        assert!(diagnostics.is_empty());
    }

    struct NoEntropy;

    impl TryRng for NoEntropy {
        type Error = io::Error;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Err(io::Error::other("no OS entropy"))
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Err(io::Error::other("no OS entropy"))
        }

        fn try_fill_bytes(&mut self, _destination: &mut [u8]) -> Result<(), Self::Error> {
            Err(io::Error::other("no OS entropy"))
        }
    }

    impl TryCryptoRng for NoEntropy {}

    #[test]
    fn propagates_seed_failure_without_a_fallback() {
        let mut diagnostics = Vec::new();
        let error = generate_seeded(25, &mut diagnostics, &mut NoEntropy).unwrap_err();
        assert_eq!(
            error.to_string(),
            "could not seed the password generator from the operating system: no OS entropy"
        );
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn warns_once_and_retries_without_disclosing_candidates() {
        let minimum = 32;
        let below = "private";
        let equal = "x".repeat(minimum);
        let above = "y".repeat(minimum + 1);
        let mut candidates = [below, &equal, &above].into_iter();
        let mut diagnostics = Vec::new();
        let password = generate_with(minimum, &mut diagnostics, || {
            Ok(move || candidates.next().unwrap().to_owned())
        })
        .unwrap();
        assert_eq!(password, above);
        let diagnostics = String::from_utf8(diagnostics).unwrap();
        for secret in [below, &equal, &above] {
            assert!(!diagnostics.contains(secret));
        }
        let mut lines = diagnostics.lines();
        assert!(
            lines
                .next()
                .unwrap()
                .contains("significantly restricts password entropy")
        );
        assert_eq!(lines.next(), None);
    }

    struct BrokenWriter;

    impl Write for BrokenWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
    }

    #[test]
    fn failed_diagnostic_writes_do_not_withhold_the_password() {
        let minimum = 32;
        let equal = "x".repeat(minimum);
        let above = "y".repeat(minimum + 1);
        let mut candidates = [equal, above.clone()].into_iter();
        let password = generate_with(minimum, &mut BrokenWriter, || {
            Ok(move || candidates.next().unwrap())
        })
        .unwrap();
        assert_eq!(password, above);
    }

    #[test]
    fn candidates_have_five_cased_dictionary_words_and_mixed_joiners() {
        let dictionary = Language::English.word_list();
        let mut rng = StdRng::from_seed([42; 32]);
        for _ in 0..128 {
            let candidate = generate_candidate(&mut rng);
            let words: Vec<_> = candidate
                .split(|letter: char| !letter.is_ascii_alphabetic())
                .collect();
            assert_eq!(words.len(), 5);
            let mut uppercase_count = 0;
            for word in words {
                assert!(dictionary.contains(&word.to_ascii_lowercase().as_str()));
                if word.bytes().all(|letter| letter.is_ascii_uppercase()) {
                    uppercase_count += 1;
                } else {
                    assert!(word.bytes().all(|letter| letter.is_ascii_lowercase()));
                }
            }
            assert!((1..=4).contains(&uppercase_count));

            let joiners: Vec<_> = candidate
                .bytes()
                .filter(|byte| !byte.is_ascii_alphabetic())
                .collect();
            assert_eq!(joiners.len(), 4);
            let digit_count = joiners.iter().filter(|byte| byte.is_ascii_digit()).count();
            assert!((1..=3).contains(&digit_count));
            assert!(
                joiners
                    .iter()
                    .all(|byte| byte.is_ascii_digit() || b"&+.,/-=?@#^%".contains(byte))
            );
        }
    }
}
