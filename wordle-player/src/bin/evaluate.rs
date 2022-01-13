use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};
use wordle_engine::WordleEngine;

use ::wordle_player::Information;

fn read_word_list(mut file: File) -> io::Result<Vec<String>> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.split('\n').map(str::to_string).collect())
}

fn get_num_tries(mut engine: WordleEngine, word_list: &[String]) -> usize {
    let mut count = 0;
    let mut info = Information::new();
    loop {
        count += 1;
        let mut allowed = word_list.iter().filter(|word| info.allows(word));
        let first = allowed.next();
        let second = allowed.next();
        if first.is_none() {
            unreachable!("No words matched");
        } else if second.is_none() {
            break count;
        } else {
            let guess = info.get_ideal_guess(word_list);
            info.update(guess, engine.guess(guess).expect("Illegal guess made"));
        }
    }
}

fn main() -> io::Result<()> {
    let word_file = File::open("../wordle-engine/popular-dedup.txt")?;
    let word_list = read_word_list(word_file)?;
    let mut bins: HashMap<usize, Vec<String>> = HashMap::new();
    for (i, word) in word_list.iter().enumerate() {
        let engine = WordleEngine::with_answer(word_list.clone(), word.to_string());
        let num_tries = get_num_tries(engine, &word_list);
        if !bins.contains_key(&num_tries) {
            println!("First word to take {} guesses: {}", num_tries, word);
        }
        bins.entry(num_tries)
            .or_insert(Vec::new())
            .push(word.to_string());
        if i % 20 == 0 {
            println!("Done word #{} ({})", i, word);
        }
    }
    let counts: HashMap<usize, usize> = bins
        .iter()
        .map(|(&count, words)| (count, words.len()))
        .collect();
    println!("{:?}", bins);
    println!("{:?}", counts);
    Ok(())
}
