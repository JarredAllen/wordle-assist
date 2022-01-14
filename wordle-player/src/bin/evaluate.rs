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

fn read_word_list(mut file: File) -> io::Result<Vec<&'static str>> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents
        .split('\n')
        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
        .filter(|s| *s != "")
        .collect())
}

fn get_num_tries(mut engine: WordleEngine, word_list: &[&str]) -> usize {
    let mut count = 0;
    let mut info = Information::new();
    let mut guess = "";
    let mut allowed = Vec::from(word_list);
    loop {
        count += 1;
        if allowed.is_empty() {
            unreachable!("No words matched, solution {}", engine.get_solution());
        } else if allowed.len() == 1 {
            if allowed[0] == guess {
                break count - 1;
            } else {
                break count;
            }
        } else {
            guess = info.get_ideal_guess_from_allowed(&allowed, word_list);
            info.update(guess, engine.guess(guess).expect("Illegal guess made"));
            allowed = allowed
                .into_iter()
                .filter(|word| info.allows(word))
                .collect();
        }
    }
}

fn main() -> io::Result<()> {
    let word_file = File::open("../wordle-engine/popular.txt")?;
    let word_list = read_word_list(word_file)?;
    let bins: RwLock<HashMap<usize, Vec<String>>> = RwLock::new(HashMap::new());
    let count = AtomicUsize::new(0);
    word_list.par_iter().for_each(|word| {
        let engine = WordleEngine::with_answer(word_list.clone(), word);
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
