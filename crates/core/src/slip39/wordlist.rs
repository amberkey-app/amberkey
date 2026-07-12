//! Official SLIP-39 wordlist (1024 words, sorted, unique 4-letter prefixes).

use std::sync::LazyLock;

static RAW: &str = include_str!("wordlist.txt");

pub static WORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let words: Vec<&str> = RAW.split_ascii_whitespace().collect();
    assert_eq!(words.len(), 1024, "SLIP-39 wordlist must contain 1024 words");
    words
});

/// Index of `word` in the wordlist (the list is sorted, so binary search).
pub fn word_index(word: &str) -> Option<u16> {
    WORDS.binary_search(&word).ok().map(|i| i as u16)
}
