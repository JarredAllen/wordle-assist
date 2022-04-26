use itertools::Itertools;
use std::fmt::Display;

use wordle_engine::{LetterResponse, WordleResponse};

/// A struct which encapsulates the guesser's knowledge
#[derive(Debug, Clone, Copy)]
pub struct Information {
    counts: [(u8, u8); 26],
    exact: [[Option<bool>; 26]; 5],
}

impl Information {
    /// Create a new struct representing no information
    pub fn new() -> Self {
        Information {
            counts: [(0, 5); 26],
            exact: [[None; 26]; 5],
        }
    }

    /// Update this to contain the information from the given guess
    pub fn update(&mut self, guess: &str, WordleResponse(response): WordleResponse) {
        for (i, (c, response)) in guess.chars().zip(response.iter()).enumerate() {
            let char_index = c as usize - 97;
            match response {
                LetterResponse::Absent => self.exact[i][char_index] = Some(false),
                LetterResponse::Misplaced => {
                    self.exact[i][char_index] = Some(false);
                }
                LetterResponse::Correct => {
                    for c in 0..26 {
                        self.exact[i][c] = Some(c == char_index);
                    }
                }
            }
        }
        for (i, count) in self.counts.iter_mut().enumerate() {
            let c = (i as u8 + 97) as char;
            let num_present = guess
                .chars()
                .zip(response.iter())
                .filter(|(ch, r)| c == *ch && **r != LetterResponse::Absent)
                .count() as u8;
            let absent = guess
                .chars()
                .zip(response.iter())
                .any(|(ch, r)| c == ch && *r == LetterResponse::Absent);
            count.0 = count.0.max(num_present);
            if absent {
                count.1 = count.1.min(num_present);
            }
        }
    }

    /// Returns whether or not this word is allowed by the current information
    pub fn allows(&self, word: &str) -> bool {
        self.exact
            .iter()
            .zip(word.chars())
            .all(|(exact, wc)| exact[wc as usize - 97] != Some(false))
            && self.counts.iter().enumerate().all(|(i, (min, max))| {
                let c = (i as u8 + 97) as char;
                let count = word.matches(c).count();
                *min as usize <= count && *max as usize >= count
            })
    }

    /// Returns the expected bits of entropy gained by this guess
    pub fn evaluate_guess(&self, word_list: &[&str], guess: &str) -> f64 {
        let allowed: Vec<&str> = word_list
            .iter()
            .filter(|word| self.allows(word))
            .cloned()
            .collect();
        self.evaluate_guess_from_allowed(&allowed, guess)
    }

    /// Like `evaluate_guess`, but the word_list must already be filtered for allowed words
    fn evaluate_guess_from_allowed(&self, word_list: &[&str], guess: &str) -> f64 {
        let mut bins = [0; 243];
        word_list
            .iter()
            .for_each(|word| bins[get_bin(guess, word)] += 1);
        let total = bins.iter().sum::<usize>() as f64;
        let start_entropy = total.log2();
        bins.into_iter()
            .filter(|&count| count != 0)
            .map(|count| {
                let count = count as f64;
                let final_entropy = count.log2();
                (start_entropy - final_entropy) * count / total
            })
            .sum()
    }

    /// Get the ideal guess from the given list of words
    pub fn get_ideal_guess<'a>(&self, word_list: &'a [&'a str]) -> &'a str {
        let allowed_words: Vec<&'a str> = word_list
            .iter()
            .filter(|word| self.allows(word))
            .cloned()
            .collect();
        self.get_ideal_guess_from_allowed(&allowed_words, word_list)
    }

    /// Get the ideal guess from the list of words, assuming the allowed words are exactly the
    /// possible words (will not have correct behavior if this isn't met).
    ///
    /// Example call:
    /// ```
    /// let info = wordle_player::Information::new();
    /// let word_list = vec!["apple", "squid", "wires"];
    /// let allowed: Vec<&str> = word_list.iter().filter(|word| info.allows(word)).cloned().collect();
    /// info.get_ideal_guess_from_allowed(&allowed, &word_list);
    /// ```
    ///
    /// This method is presented as a potential performance optimization if repeated calls are
    /// made with the same information and the same word list.
    pub fn get_ideal_guess_from_allowed<'a>(
        &self,
        allowed_words: &[&str],
        word_list: &[&'a str],
    ) -> &'a str {
        word_list
            .iter()
            .map(|word| {
                (
                    word,
                    self.evaluate_guess_from_allowed(allowed_words, word),
                    self.allows(word),
                )
            })
            // We pick the word which gives us the most information,
            // breaking ties first by picking a word in the list, then
            // by picking the word which is last alphabetically.
            .max_by(|(_, s1, w1_in), (_, s2, w2_in)| {
                s1.partial_cmp(s2)
                    .expect("Unexpected NaN :(")
                    .then_with(|| {
                        use std::cmp::Ordering;
                        match (w1_in, w2_in) {
                            (false, true) => Ordering::Less,
                            (true, false) => Ordering::Greater,
                            _ => Ordering::Equal,
                        }
                    })
            })
            .expect("Empty word list :(")
            .0
    }

    pub fn top_n_guesses<'a>(
        &self,
        guess_list: &[&'a str],
        word_list: &[&'a str],
        count: usize,
    ) -> Vec<(&'a str, f64)> {
        let allowed_words: Vec<&'a str> = word_list
            .iter()
            .filter(|word| self.allows(word))
            .cloned()
            .collect();
        guess_list
            .iter()
            .map(|word| {
                (
                    word,
                    self.evaluate_guess_from_allowed(&allowed_words, word),
                    allowed_words.contains(word),
                )
            })
            // We pick the word which gives us the most information,
            // breaking ties first by picking a word in the list, then
            // by picking the word which is last alphabetically.
            .sorted_unstable_by(|(_, s1, w1_in), (_, s2, w2_in)| {
                s1.partial_cmp(s2)
                    .expect("Unexpected NaN :(")
                    .then_with(|| {
                        use std::cmp::Ordering;
                        match (w1_in, w2_in) {
                            (false, true) => Ordering::Less,
                            (true, false) => Ordering::Greater,
                            _ => Ordering::Equal,
                        }
                    })
                    .reverse()
            })
            .take(count)
            .map(|(word, score, _)| (*word, score))
            .collect()
    }
}

impl Display for Information {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Exact:")?;
        for slot in self.exact {
            if let Some((i, _)) = slot.iter().enumerate().find(|(_, c)| **c == Some(true)) {
                let c = (i as u8 + 97) as char;
                writeln!(f, "\t[ {} ]", c)?;
            } else {
                writeln!(
                    f,
                    "\t[ {} ]",
                    slot.iter()
                        .enumerate()
                        .filter(|(_, c)| **c == Some(false))
                        .map(|(c, _)| format!("!{}", (c as u8 + 97) as char))
                        .collect::<Vec<String>>()
                        .join(", ")
                )?;
            }
        }
        writeln!(f, "Counts:")?;
        for (i, (min, max)) in self.counts.iter().enumerate() {
            let c = (i as u8 + 97) as char;
            writeln!(f, "{}: Between {} and {}", c, min, max)?;
        }
        Ok(())
    }
}

impl Default for Information {
    fn default() -> Self {
        Self::new()
    }
}

fn get_bin(guess: &str, word: &str) -> usize {
    let mut response = [LetterResponse::Absent; 5];
    let mut taken = [false; 5];
    guess
        .chars()
        .zip(word.chars())
        .enumerate()
        .for_each(|(i, (gc, wc))| {
            if wc == gc {
                response[i] = LetterResponse::Correct;
                taken[i] = true;
            }
        });
    for (word_char, response) in guess.chars().zip(response.iter_mut()) {
        if *response == LetterResponse::Correct {
            continue;
        }
        for (solution_char, taken) in word.chars().zip(taken.iter_mut()) {
            if !*taken && word_char == solution_char {
                *taken = true;
                *response = LetterResponse::Misplaced;
                break;
            }
        }
    }
    response.into_iter().fold(0, |acc, resp| {
        acc * 3
            + match resp {
                LetterResponse::Absent => 0,
                LetterResponse::Misplaced => 1,
                LetterResponse::Correct => 2,
            }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORD_LIST: &'static [&'static str] = &["aegis", "favor", "wired", "weird"];
    #[test]
    /// Tests that no information means all words allowed
    fn test_allow_default() {
        let word_list: Vec<String> = WORD_LIST.iter().cloned().map(str::to_string).collect();
        let info = Information::new();
        for word in &word_list {
            assert!(info.allows(word));
        }
    }

    #[test]
    fn test_binning() {
        assert_eq!(get_bin("abbey", "abbey"), 242);
    }

    #[test]
    fn test_duplicate_letters() {
        use LetterResponse::*;
        let mut info = Information::new();
        info.update(
            "bibbs",
            WordleResponse([Misplaced, Absent, Correct, Absent, Absent]),
        );
        assert!(info.allows("abbey"));
        assert!(!info.allows("abbes"));
    }
}
