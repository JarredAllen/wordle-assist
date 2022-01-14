use itertools::Itertools;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};
use wordle_engine::{WordleEngine, WordleResponse};

use ::wordle_player::Information;

fn read_word_list(mut file: File) -> io::Result<Vec<&'static str>> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents
        .split('\n')
        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
        .filter(|s| !s.is_empty())
        .collect())
}

fn get_num_guesses_for_words<'a, 'b>(
    word_list: &'b [&'b str],
    remaining: Vec<&'a str>,
) -> Box<dyn Iterator<Item = (&'a str, usize)> + 'b>
where
    'a: 'b,
{
    if remaining.len() <= 1 {
        Box::new(remaining.into_iter().map(|word| (word, 1)))
    } else {
        let guess = Information::new()
            .get_ideal_guess_from_allowed(&remaining, word_list)
            .to_string();
        Box::new(
            WordleResponse::all_responses()
                .map(move |response| {
                    (
                        response,
                        remaining
                            .iter()
                            .filter(|word| WordleEngine::get_response(word, &guess) == response)
                            .cloned()
                            .collect::<Vec<&'a str>>(),
                    )
                })
                .filter(|(_, v)| !v.is_empty())
                .flat_map(|(r, v)| {
                    get_num_guesses_for_words(word_list, v).map(move |(word, count)| {
                        (
                            word,
                            count + if r == WordleResponse::correct() { 0 } else { 1 },
                        )
                    })
                }),
        )
    }
}

fn main() -> io::Result<()> {
    let word_file = File::open("../wordle-engine/scrabble.txt")?;
    let word_list = read_word_list(word_file)?;
    let mut bins: HashMap<usize, Vec<&'static str>> = HashMap::new();
    for (word, num_guesses) in get_num_guesses_for_words(&word_list, word_list.clone()) {
        bins.entry(num_guesses).or_default().push(word);
    }
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
