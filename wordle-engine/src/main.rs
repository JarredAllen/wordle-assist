use ::wordle_engine::{LetterResponse, WordleEngine, WordleResponse};
use std::fs::File;
use std::io::{self, Read};

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
    let solution_list = read_word_list("possible-answers.txt")?;
    let guess_list = read_word_list("possible-guesses.txt")?;
    let mut engine = WordleEngine::new(guess_list, solution_list);
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
