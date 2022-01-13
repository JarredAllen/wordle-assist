use itertools::Itertools;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    sync::{
        atomic::{AtomicUsize, Ordering},
        RwLock,
    },
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
    let mut guess = "";
    loop {
        count += 1;
        let mut allowed = word_list.iter().filter(|word| info.allows(word));
        let first = allowed.next();
        let second = allowed.next();
        let first = first.unwrap_or_else(|| unreachable!("No words matched"));
        if second.is_none() {
            if first == guess {
                break count - 1;
            } else {
                break count;
            }
        } else {
            guess = info.get_ideal_guess(word_list);
            info.update(guess, engine.guess(guess).expect("Illegal guess made"));
        }
    }
}

fn main() -> io::Result<()> {
    let word_file = File::open("../wordle-engine/scrabble-dedup.txt")?;
    let word_list = read_word_list(word_file)?;
    let bins: RwLock<HashMap<usize, Vec<String>>> = RwLock::new(HashMap::new());
    let count = AtomicUsize::new(0);
    word_list.par_iter().for_each(|word| {
        let engine = WordleEngine::with_answer(word_list.clone(), word.to_string());
        let num_tries = get_num_tries(engine, &word_list);
        // Print status info so user knows it's running
        let index = count.fetch_add(1, Ordering::Relaxed) + 1;
        if index % 20 == 0 {
            eprintln!("Word #{}/{} done ({})", index, word_list.len(), word);
        }
        // Write actual data to hash map
        bins.write()
            .expect("RwLock fail")
            .entry(num_tries)
            .or_insert(Vec::new())
            .push(word.to_string());
    });
    // Write final output data
    let bins = bins.into_inner().expect("RwLock fail");
    let counts: HashMap<usize, usize> = bins
        .iter()
        .map(|(&count, words)| (count, words.len()))
        .collect();
    println!("Counts: {:?}", counts);
    for tries in bins.keys().sorted() {
        let mut words = Vec::new();
        words.extend(bins[tries].iter().cloned());
        words.sort_unstable();
        println!("Words that took {} guesses:\n{:?}", tries, words);
    }
    Ok(())
}
