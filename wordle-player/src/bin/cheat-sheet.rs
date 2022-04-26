use std::{
    fs::File,
    io::{self, Read},
};
use wordle_engine::{WordleEngine, WordleResponse};

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

fn find_word_paths<'a>(
    word_list: &'a [&'a str],
    remaining: Vec<&'a str>,
) -> Box<dyn Iterator<Item = (&'a str, Vec<&'a str>)> + 'a> {
    if remaining.len() <= 1 {
        Box::new(remaining.into_iter().map(|word| (word, vec![word])))
    } else {
        let guess = Information::new().get_ideal_guess_from_allowed(&remaining, word_list);
        Box::new(
            WordleResponse::all_responses()
                .map(move |response| {
                    (
                        response,
                        remaining
                            .iter()
                            .filter(|word| WordleEngine::get_response(word, guess) == response)
                            .cloned()
                            .collect::<Vec<&'a str>>(),
                    )
                })
                .filter(|(_, v)| !v.is_empty())
                .flat_map(move |(r, v)| {
                    find_word_paths(word_list, v).map(move |(word, path)| {
                        (
                            word,
                            if r == WordleResponse::correct() {
                                path
                            } else {
                                path.into_iter()
                                    .rev()
                                    .chain([guess].into_iter())
                                    .rev()
                                    .collect()
                            },
                        )
                    })
                }),
        )
    }
}

fn main() -> io::Result<()> {
    let solution_list = read_word_list("../wordle-engine/possible-answers.txt")?;
    let guess_list = read_word_list("..//wordle-engine/possible-guesses.txt")?;
    for (word, path) in find_word_paths(&guess_list, solution_list) {
        println!("{}: {}", word, path.join(" -> "));
    }
    Ok(())
}
