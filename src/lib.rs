//! Spelling corrector in Rust.
//! The implementation is based on [Peter Norvig's essay](http://norvig.com/spell-correct.html).

#![warn(clippy::all, clippy::pedantic, missing_docs)]

use std::collections::HashMap;
use std::collections::HashSet;

use regex::Regex;

/// `SpellingCorrector` is a type that represents a spelling corrector.
pub struct SpellingCorrector<'a> {
    /// An alphabet used by text data.
    pub alphabet: &'a str,
    /// A frequency table storing frequencies of words from text data.
    pub freqmap: HashMap<String, u32>,
}

impl<'a> SpellingCorrector<'a> {
    /// `new` creates a new `SpellingCorrector` with an English alphabet.
    ///
    /// # Errors
    ///
    /// Returns error if `std::fs::read_to_string` fails or if an invalid expression is given to
    /// `regex::Regex::new`.
    ///
    /// # Arguments
    ///
    /// * `path` - A path to text data.
    ///
    /// # Example
    ///
    /// ```
    /// use spell::SpellingCorrector;
    ///
    /// fn main() -> Result<(), anyhow::Error> {
    ///     let sc = SpellingCorrector::new("data/big.txt")?;
    ///     assert_eq!(sc.freqmap.len(), 32_198);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(path: &'a str) -> Result<Self, anyhow::Error> {
        Self::with_alphabet(path, "abcdefghijklmnopqrstuvwxyz")
    }

    /// `with_alphabet` creates a new `SpellingCorrector` with a user-specified alphabet.
    ///
    /// # Errors
    ///
    /// Returns error if `std::fs::read_to_string` fails or if an invalid expression is given to
    /// `regex::Regex::new`.
    ///
    /// # Arguments
    ///
    /// * `path` - A path to text data.
    /// * `alphabet` - An alphabet used by text data.
    ///
    /// # Example
    ///
    /// ```
    /// use spell::SpellingCorrector;
    ///
    /// fn main() -> Result<(), anyhow::Error> {
    ///     let alphabet = "abcdefghijklmnopqrstuvwxyz";
    ///     let sc = SpellingCorrector::with_alphabet("data/big.txt", alphabet)?;
    ///     assert_eq!(sc.freqmap.len(), 32_198);
    ///     Ok(())
    /// }
    /// ```
    pub fn with_alphabet(path: &'a str, alphabet: &'a str) -> Result<Self, anyhow::Error> {
        let text = std::fs::read_to_string(path)?;

        let mut freqmap = HashMap::new();
        for word in Regex::new(r"\w+")?.find_iter(&text) {
            *freqmap.entry(word.as_str().to_lowercase()).or_insert(0) += 1;
        }

        Ok(Self { alphabet, freqmap })
    }

    /// `correction` computes the most probable spelling correction for `word`.
    ///
    /// # Panics
    ///
    /// Never panics.
    ///
    /// # Arguments
    ///
    /// * `word` - A word.
    ///
    /// # Example
    ///
    /// ```
    /// use spell::SpellingCorrector;
    ///
    /// fn main() -> Result<(), anyhow::Error> {
    ///     let sc = SpellingCorrector::new("data/big.txt")?;
    ///     let c = sc.correction("speling");
    ///     assert_eq!(c, "spelling");
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn correction(&self, word: &str) -> String {
        self.candidates(word)
            .into_iter()
            // SAFETY: All `a`s and `b`s are at most `4_294_967_295` (i.e., `u64::pow(2, 32) - 1`)
            .max_by(|a, b| self.p(a).partial_cmp(&self.p(b)).unwrap())
            // SAFETY: `self.candidates(word)` always contains at least one element
            .unwrap()
    }

    /// `candidates` generates possible spelling corrections for `word`.
    ///
    /// # Arguments
    ///
    /// * `word` - A word.
    #[must_use]
    fn candidates(&self, word: &str) -> HashSet<String> {
        let k1 = self.known(vec![String::from(word)]);
        if !k1.is_empty() {
            return k1;
        }

        let k2 = self.known(self.edits1(word));
        if !k2.is_empty() {
            return k2;
        }

        let k3 = self.known(self.edits2(word));
        if !k3.is_empty() {
            return k3;
        }

        HashSet::from_iter(vec![String::from(word)])
    }

    /// `p` computes a probability of `word`.
    ///
    /// # Arguments
    ///
    /// * `word` - A word.
    #[must_use]
    fn p(&self, word: &str) -> f64 {
        f64::from(self.freqmap[word]) / f64::from(self.freqmap.values().sum::<u32>())
    }

    /// `known` computes the subset of `words` that appear in `freqmap`.
    ///
    /// # Arguments
    ///
    /// * `words` - A vector of words.
    #[must_use]
    fn known(&self, words: impl IntoIterator<Item = String>) -> HashSet<String> {
        words
            .into_iter()
            .filter(|word| self.freqmap.contains_key(word))
            .collect()
    }

    /// `edits1` computes all edits that are one edit away from `word`.
    ///
    /// # Arguments
    ///
    /// * `word` - A word.
    #[must_use]
    fn edits1(&self, word: &str) -> HashSet<String> {
        let splits = (0..=word.len())
            .map(|i| (&word[..i], &word[i..]))
            .collect::<Vec<(&str, &str)>>();

        let deletes = splits
            .iter()
            .filter(|(_, r)| !r.is_empty())
            .map(|(l, r)| (*l).to_string() + &r[1..]);

        let transposes = splits
            .iter()
            .filter(|(_, r)| r.len() > 1)
            .map(|(l, r)| (*l).to_string() + &r[1..2] + &r[0..1] + &r[2..]);

        let replaces = splits
            .iter()
            .filter(|(_, r)| !r.is_empty())
            .flat_map(|(l, r)| {
                self.alphabet
                    .chars()
                    .map(|c| (*l).to_string() + &c.to_string() + &r[1..])
            });

        let inserts = splits.iter().flat_map(|(l, r)| {
            self.alphabet
                .chars()
                .map(|c| (*l).to_string() + &c.to_string() + r)
        });

        deletes
            .chain(transposes)
            .chain(replaces)
            .chain(inserts)
            .collect()
    }

    /// `edits2` computes all edits that are two edits away from `word`.
    ///
    /// # Arguments
    ///
    /// * `word` - A word.
    #[must_use]
    fn edits2(&self, word: &str) -> Vec<String> {
        self.edits1(word)
            .into_iter()
            .flat_map(|e1| self.edits1(&e1))
            .collect()
    }
}
