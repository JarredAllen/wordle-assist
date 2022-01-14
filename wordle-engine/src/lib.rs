use rand::{self, seq::SliceRandom};

/// An engine for playing Wordle
pub struct WordleEngine {
    word_list: Vec<&'static str>,
    solution: &'static str,
    state: [LetterStatus; 5],
}

/// The status of a letter
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(unused)]
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
    pub fn new(word_list: Vec<&'static str>) -> Self {
        let solution: &'static str = word_list
            .choose(&mut rand::thread_rng())
            .expect("Empty word list");
        Self::with_answer(word_list, solution)
    }

    /// Create a new WordleEngine instance with the given word list and given solution
    pub fn with_answer(word_list: Vec<&'static str>, solution: &'static str) -> Self {
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
            let mut taken = [false; 5];
            word.chars()
                .zip(self.solution.chars())
                .enumerate()
                .for_each(|(i, (wc, sc))| {
                    if wc == sc {
                        self.state[i] = LetterStatus::Exact;
                        response[i] = LetterResponse::Correct;
                        taken[i] = true;
                    }
                });
            for (word_char, response) in word.chars().zip(response.iter_mut()) {
                if *response == LetterResponse::Correct {
                    continue;
                }
                for (solution_char, taken) in self.solution.chars().zip(taken.iter_mut()) {
                    if !*taken && word_char == solution_char {
                        *taken = true;
                        *response = LetterResponse::Misplaced;
                        break;
                    }
                }
            }
            Some(WordleResponse(response))
        } else {
            None
        }
    }

    pub fn solved(&self) -> bool {
        self.state == [LetterStatus::Exact; 5]
    }

    pub fn get_solution(&self) -> &str {
        self.solution
    }

    /// Returns true iff the word is legal to guess
    fn can_guess(&self, word: &str) -> bool {
        self.word_list.iter().any(|&w| w == word)
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
