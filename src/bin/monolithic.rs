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

    let mut word_freqs: Vec<(String, usize)> = Vec::new();

    // open input file
    let f = File::open(filename).expect("Unable to open input file for reading.");
    for line in BufReader::new(f).lines() {
        let line = line.unwrap();
        let line = line.as_str();
        for word in line.unicode_words() {
            let word = word.to_lowercase();
            if stop_words.contains(&word.as_str()) {
                continue;
            }
            let mut found = false;
            for pair in word_freqs.iter_mut() {
                if word == pair.0 {
                    pair.1 += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                word_freqs.push((word, 1));
            }
        }
    }

    word_freqs.sort_by(|a, b| b.1.cmp(&a.1));
    for tf in word_freqs.iter().take(25) {
        println!("{tf:?}");
    }
    Ok(())
}
