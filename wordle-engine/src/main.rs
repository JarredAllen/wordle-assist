use ::wordle_engine::{LetterResponse, WordleEngine, WordleResponse};
use std::fs::File;
use std::io::{self, Read};

fn read_word_list(mut file: File) -> io::Result<Vec<String>> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.split('\n').map(str::to_string).collect())
}

fn main() -> io::Result<()> {
    let word_file = File::open("scrabble-dedup.txt")?;
    let word_list = read_word_list(word_file)?;
    let mut engine = WordleEngine::new(word_list);
    let mut guess = String::new();
    while !engine.solved() {
        println!("Please make a guess (leave blank to forfeit):");
        guess.clear();
        io::stdin().read_line(&mut guess)?;
        if guess == "\n" {
            println!("You gave up :(");
            println!("The answer was {}", engine.get_solution());
            break;
        }
        match engine.guess(guess.trim()) {
            None => println!("Illegal guess"),
            Some(WordleResponse(arr)) => println!(
                "{}",
                arr.iter()
                    .map(|c| match c {
                        LetterResponse::Absent => '.',
                        LetterResponse::Misplaced => '?',
                        LetterResponse::Correct => '!',
                    })
                    .fold(String::new(), |mut a, b| {
                        a.push(b);
                        a
                    })
            ),
        }
    }
    Ok(())
}
