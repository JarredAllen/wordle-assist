use rand::{self, seq::SliceRandom};

/// An engine for playing Wordle
pub struct WordleEngine {
    word_list: Vec<String>,
    solution: String,
    state: [LetterStatus; 5],
}

/// The status of a letter
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LetterStatus {
    /// Exactly known
    Exact,
    /// Known that it is present, but not where
    Present,
    /// Not known
    Unknown,
}

impl WordleEngine {
    /// Create a new WordleEngine instance from the given list, with a random word
    pub fn new(word_list: Vec<String>) -> Self {
        let solution = word_list
            .choose(&mut rand::thread_rng())
            .expect("Empty word list")
            .clone();
        Self::with_answer(word_list, solution)
    }

    /// Create a new WordleEngine instance with the given word list and given solution
    pub fn with_answer(word_list: Vec<String>, solution: String) -> Self {
        WordleEngine {
            word_list,
            solution,
            state: [LetterStatus::Unknown; 5],
        }
    }

    /// Try to make a given guess. Returns:
    ///  - `None` if the guess is invalid
    ///  - `Some(response)` if the guess is valid
    pub fn guess(&mut self, word: &str) -> Option<WordleResponse> {
        if word == self.solution {
            self.state = [LetterStatus::Exact; 5];
            Some(WordleResponse::correct())
        } else if self.can_guess(word) {
            let mut response = [LetterResponse::Absent; 5];
            word.chars()
                .zip(self.solution.chars())
                .enumerate()
                .for_each(|(i, (wc, sc))| {
                    if wc == sc {
                        self.state[i] = LetterStatus::Exact;
                        response[i] = LetterResponse::Correct;
                    } else if let Some(index) = word.find(sc) {
                        // TODO handle words with duplicate letters
                        self.state[i] = LetterStatus::Present;
                        response[index] = LetterResponse::Misplaced;
                    }
                });
            Some(WordleResponse(response))
        } else {
            None
        }
    }

    pub fn solved(&self) -> bool {
        self.state == [LetterStatus::Exact; 5]
    }

    /// Returns true iff the word is legal to guess
    fn can_guess(&self, word: &str) -> bool {
        self.word_list.iter().any(|w| w == word)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct WordleResponse(pub [LetterResponse; 5]);
impl WordleResponse {
    fn correct() -> Self {
        Self([LetterResponse::Correct; 5])
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LetterResponse {
    Correct,
    Misplaced,
    Absent,
}
