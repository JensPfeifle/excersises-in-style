use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, Read};
use std::io::{BufReader, Result};
use unicode_segmentation::UnicodeSegmentation;

fn main() -> Result<()> {
    // parse args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing required argument: input file path.");
        return Ok(());
    }
    let filename = &args[1];

    // Load stop words
    let mut stop_words_file =
        File::open("./stop_words.txt").expect("Unable to open stop words file for reading.");
    let mut buffer = String::new();
    stop_words_file
        .read_to_string(&mut buffer)
        .expect("Error reading from stop words file.");
    let stop_words: Vec<&str> = buffer.split(',').collect();

    // create iterator over words from input file
    let f = File::open(filename).expect("Unable to open input file for reading.");
    let words = BufReader::new(f)
        .lines()
        .map(|line| line.unwrap())
        .map(|line| {
            line.unicode_words()
                .map(|word| word.to_lowercase())
                .filter(|word| !stop_words.contains(&word.as_str()))
                .collect::<Vec<String>>()
        })
        .flatten();

    let mut word_freqs: BTreeMap<String, usize> = BTreeMap::new();
    for word in words {
        *word_freqs.entry(word).or_insert(0) += 1;
    }

    let mut pairs: Vec<(&String, &usize)> = word_freqs.iter().collect();
    pairs.sort_by(|a, b| a.1.cmp(b.1).reverse());
    for pair in pairs.iter().take(25) {
        println!("{pair:?}");
    }
    Ok(())
}
