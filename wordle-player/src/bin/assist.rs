use std::fs::File;
use std::io::{self, Read};
use wordle_engine::{LetterResponse, WordleResponse};

use ::wordle_player::Information;

fn read_word_list(mut file: File) -> io::Result<Vec<String>> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.split('\n').map(str::to_string).collect())
}

fn main() -> io::Result<()> {
    let word_file = File::open("../wordle-engine/scrabble-dedup.txt")?;
    let word_list = read_word_list(word_file)?;
    let mut info = Information::new();
    let mut guess = String::new();
    let mut response = String::new();
    loop {
        let allowed: Vec<String> = word_list
            .iter()
            .filter(|word| info.allows(word))
            .cloned()
            .collect();
        if allowed.len() == 1 {
            println!("Answer: {}", allowed[0]);
            break;
        } else if allowed.is_empty() {
            println!("No words match information:");
            println!("{:?}", info);
            break;
        }
        let best_guess = info.get_ideal_guess(&word_list);
        let score = info.evaluate_guess(&allowed, best_guess);
        println!("Recommended guess: {} (+{})", best_guess, score);
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
