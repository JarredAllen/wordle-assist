use std::fs::File;
use std::io::{self, Read};
use wordle_engine::{LetterResponse, WordleResponse};

use ::wordle_player::Information;

fn read_word_list(filename: &str) -> io::Result<Vec<&'static str>> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents
        .split('\n')
        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
        .filter(|s| !s.is_empty())
        .collect())
}

fn main() -> io::Result<()> {
    let mut allowed = read_word_list("../wordle-engine/possible-answers.txt")?;
    let guess_list = read_word_list("../wordle-engine/possible-guesses.txt")?;
    let mut info = Information::new();
    let mut guess = String::new();
    let mut response = String::new();
    loop {
        allowed = allowed
            .iter()
            .filter(|word| info.allows(word))
            .cloned()
            .collect();
        if allowed.len() == 1 {
            println!("Answer: {}", allowed[0]);
            break;
        } else if allowed.is_empty() {
            println!("No words match information:");
            println!("{}", info);
            break;
        }
        println!(
            "Top 5 guesses: [{}]",
            info.top_n_guesses(&guess_list, &allowed, 5)
                .into_iter()
                .map(|(word, score)| format!("({}, {:.5})", word, score))
                .collect::<Vec<String>>()
                .join(", ")
        );
        if allowed.len() > 10 {
            println!("{} words remain", allowed.len());
        } else {
            println!("Remaining words: {:?}", allowed);
        }
        println!("What was your guess?");
        guess.clear();
        response.clear();
        io::stdin().read_line(&mut guess)?;
        let guess = guess.trim();
        println!("What was the response?");
        io::stdin().read_line(&mut response)?;
        let response = response.trim();
        println!(
            "You guessed {} (+{})",
            guess,
            info.evaluate_guess(&allowed, guess)
        );
        let mut letters = [LetterResponse::Absent; 5];
        for (i, c) in response.chars().enumerate() {
            match c {
                '.' => letters[i] = LetterResponse::Absent,
                '?' => letters[i] = LetterResponse::Misplaced,
                '!' => letters[i] = LetterResponse::Correct,
                _ => panic!(),
            }
        }
        info.update(guess, WordleResponse(letters));
    }
    Ok(())
}
