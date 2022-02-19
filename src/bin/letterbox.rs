use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use unicode_segmentation::UnicodeSegmentation;

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = Sender<Result<T>>;

type WordList = Vec<String>;
type WordCounts = Vec<(String, usize)>;

#[derive(Debug)]
enum FileLoaderCommand {
    LoadInput {
        filename: String,
        resp: Responder<WordList>,
    },
    LoadStopWords {
        filename: String,
        resp: Responder<WordList>,
    },
}

fn fileloader(inbox: Receiver<FileLoaderCommand>) {
    while let Ok(msg) = inbox.recv() {
        match msg {
            FileLoaderCommand::LoadInput { filename, resp } => {
                let mut f = File::open(filename).expect("Unable to open input file for reading.");
                let mut buffer = String::new();
                f.read_to_string(&mut buffer)
                    .expect("Failed to read input file");
                let words = buffer
                    .unicode_words()
                    .map(|word| word.to_lowercase())
                    .map(|word| word.to_owned())
                    .collect::<Vec<String>>();
                resp.send(Ok(words.clone()));
            }
            FileLoaderCommand::LoadStopWords { filename, resp } => {
                let mut f =
                    File::open(filename).expect("Unable to open stop words file for reading.");
                let mut buffer = String::new();
                f.read_to_string(&mut buffer)
                    .expect("Failed to read stop words file");
                let words = buffer
                    .split(',')
                    .map(|word| word.to_lowercase())
                    .map(|word| word.to_owned())
                    .collect::<Vec<String>>();
                resp.send(Ok(words.clone()));
            }
        }
    }
}

#[derive(Debug)]
enum WordCounterCommand {
    CountWords {
        words: WordList,
        stopwords: WordList,
        resp: Responder<WordCounts>,
    },
}

fn wordcounter(inbox: Receiver<WordCounterCommand>) {
    while let Ok(msg) = inbox.recv() {
        match msg {
            WordCounterCommand::CountWords {
                words,
                stopwords: stop_words,
                resp,
            } => {
                let mut word_freqs: Vec<(String, usize)> = Vec::new();
                for word in words.into_iter() {
                    if stop_words.contains(&word) {
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
                resp.send(Ok(word_freqs));
            }
        }
    }
}
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing required argument: input file path.");
        return Ok(());
    }
    let filename = args[1].to_owned();
    let (loader_tx, loader_rx) = channel();
    let (loader_resp_tx, loader_resp_rx) = channel();

    let (counter_tx, counter_rx) = channel();
    let (counter_resp_tx, counter_resp_rx) = channel();

    // spawn worker
    thread::spawn(move || fileloader(loader_rx));
    thread::spawn(move || wordcounter(counter_rx));

    // send command to trigger file load, wait for response
    loader_tx.send(FileLoaderCommand::LoadInput {
        filename: filename.clone(),
        resp: loader_resp_tx.clone(),
    });
    let input_words = loader_resp_rx.recv().unwrap().unwrap();

    // reuse file loader thread to load stop words
    loader_tx.send(FileLoaderCommand::LoadStopWords {
        filename: "./stop_words.txt".to_owned(),
        resp: loader_resp_tx.clone(),
    });
    let stop_words = loader_resp_rx.recv().unwrap().unwrap();

    // now count
    counter_tx.send(WordCounterCommand::CountWords {
        words: input_words,
        stopwords: stop_words,
        resp: counter_resp_tx.clone(),
    });

    // cheat and do this on the main thread ;)
    let mut word_freqs = counter_resp_rx.recv().unwrap().unwrap();
    word_freqs.sort_by(|a, b| b.1.cmp(&a.1));
    for tf in word_freqs.iter().take(25) {
        println!("{tf:?}");
    }
    Ok(())
}
