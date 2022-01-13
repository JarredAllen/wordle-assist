use wordle_engine::{LetterResponse, WordleResponse};

/// A struct which encapsulates the guesser's knowledge
#[derive(Debug, Clone, Copy)]
pub struct Information {
    present: [Option<bool>; 26],
    exact: [[Option<bool>; 26]; 5],
}

impl Information {
    /// Create a new struct representing no information
    pub fn new() -> Self {
        Information {
            present: [None; 26],
            exact: [[None; 26]; 5],
        }
    }

    /// Update this to contain the information from the given guess
    pub fn update(&mut self, guess: &str, WordleResponse(response): WordleResponse) {
        for (i, (c, response)) in guess.chars().zip(response.iter()).enumerate() {
            let char_index = c as usize - 97;
            match response {
                LetterResponse::Absent => {
                    self.present[char_index] = Some(false);
                    for j in 0..5 {
                        self.exact[j][char_index] = Some(false)
                    }
                }
                LetterResponse::Misplaced => {
                    self.present[char_index] = Some(true);
                    self.exact[i][char_index] = Some(false);
                }
                LetterResponse::Correct => {
                    self.present[char_index] = Some(true);
                    for c in 0..26 {
                        self.exact[i][c] = Some(c == char_index);
                    }
                }
            }
        }
    }

    /// Returns whether or not this word is allowed by the current knowledge
    pub fn allows(&self, word: &str) -> bool {
        self.present.iter().enumerate().all(|(i, present)| {
            let c = char::from(i as u8 + 97);
            present.map_or(true, |present| present == word.contains(c))
        }) && self
            .exact
            .iter()
            .zip(word.chars())
            .all(|(exact, wc)| exact[wc as usize - 97] != Some(false))
    }

    /// Returns the expected bits of entropy gained by this guess
    pub fn evaluate_guess(&self, word_list: &[String], guess: &str) -> f64 {
        let mut bins = [0; 243];
        word_list
            .iter()
            .filter(|word| self.allows(word))
            .for_each(|word| bins[get_bin(guess, word)] += 1);
        let total = word_list.iter().filter(|word| self.allows(word)).count() as f64;
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

    pub fn get_ideal_guess<'a>(&self, word_list: &'a [String]) -> &'a str {
        word_list
            .iter()
            .map(|word| (word, self.evaluate_guess(word_list, word)))
            .max_by(|(_, s1), (_, s2)| s1.partial_cmp(s2).expect("Unexpected NaN :("))
            .expect("Empty word list :(")
            .0
    }
}

impl Default for Information {
    fn default() -> Self {
        Self::new()
    }
}

fn get_bin(guess: &str, word: &str) -> usize {
    let mut acc = 0;
    for (gc, wc) in guess.chars().zip(word.chars()) {
        acc *= 3;
        if gc == wc {
            acc += 2
        } else if word.contains(gc) {
            // TODO handle words with duplicate letters
            acc += 1
        }
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORD_LIST: &'static [&'static str] = &["apple", "favor", "wired", "weird"];
    #[test]
    /// Tests that no information means all words allowed
    fn test_allow_default() {
        let word_list: Vec<String> = WORD_LIST.iter().cloned().map(str::to_string).collect();
        let info = Information::new();
        for word in &word_list {
            assert!(info.allows(word));
        }
    }
}