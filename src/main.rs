use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::time::Instant;
extern crate rayon;
use rayon::prelude::*;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

fn squential(paths: Vec<DirEntry>) -> HashMap<String, i32> {
    //Initalize new hashmap
    let mut wordcount: HashMap<String, i32> = HashMap::new();
    //loop through all book directories in paths
    for path in paths {
        //read contentes of path
        let contents = fs::read_to_string(path.path()).expect("Error");
        //loops through all words in contents
        for word in contents.split_whitespace() {
            //checks if hashmap contains word if not insert and set to 1
            *wordcount.entry(word.to_string()).or_insert(1) += 1;
        }
    }
    wordcount
}

fn parallelism(paths: Vec<DirEntry>) -> HashMap<String, i32> {
    paths
        //uses parallel iterators
        .into_par_iter()
        .map(|entry| {
            //read contentes of path
            let contents = fs::read_to_string(entry.path()).expect("Error");
            //Initalize new hashmap
            let mut wordcount: HashMap<String, i32> = HashMap::new();
            //loops through all words in contents
            for word in contents.split_whitespace() {
                //checks if hashmap contains word if not insert and set to 1
                *wordcount.entry(word.to_string()).or_insert(1) += 1;
            }
            wordcount
        })
        //combine all hash maps into one
        .reduce(
            || HashMap::new(),
            |mut acc, map| {
                //loops through all contents of map
                for (word, count) in map {
                    //accumulates all hashs into one
                    *acc.entry(word).or_insert(1) += count;
                }
                acc
            },
        )
}

//Creates message enum that allows communication between actor models
enum Message {
    FileContents(Vec<String>),
    WordCounts(Vec<HashMap<String, i32>>),
    CombinedCount(HashMap<String, i32>),
    Shutdown,
}

struct ReadActor {
    sender: Sender<Message>,
}

//sends
impl ReadActor {
    //Function that reads contents of path
    fn read(&self, paths: Vec<DirEntry>) {
        let contents: Vec<String> = paths
            //uses parallel iterators
            .into_par_iter()
            .map(|entry| fs::read_to_string(entry.path()).unwrap_or_default())
            .collect();
        self.sender.send(Message::FileContents(contents.clone())).unwrap();
    }
}

struct CountActor {
    sender: Sender<Message>,
}

impl CountActor {
    fn count(&self, words: Vec<String>) {
        println!("CountActor received message");
        let wordcount: Vec<HashMap<String, i32>> = words
            //uses parallel iterators
            .into_par_iter()
            .map(|words| {
                //Initalize new hashmap
                let mut wordcount: HashMap<String, i32> = HashMap::new();
                //loops through all words in word
                for word in words.split_whitespace() {
                    //checks if hashmap contains word if not insert and set to 1
                    *wordcount.entry(word.to_string()).or_insert(1) += 1;
                }
                wordcount
            })
            .collect();
        println!("CountActor: Sending WordCounts message.");
        self.sender.send(Message::WordCounts(wordcount)).unwrap();
    }
}

struct CombineActor {
    sender: Sender<Message>,
}

impl CombineActor {
    fn combine(&self, hashs: Vec<HashMap<String, i32>>) {
        println!("CombineActor received message");
        let combined: HashMap<String, i32> =
            hashs.into_iter().fold(HashMap::new(), |mut acc, map| {
                //loops through all contents of map
                for (word, count) in map {
                    //accumulates all hashs into one
                    *acc.entry(word).or_insert(1) += count;
                }
                acc
            });
        println!("CombineActor: Sending CombinedCount message.");
        self.sender.send(Message::CombinedCount(combined)).unwrap();
    }
}

fn pipelineparalelism(paths: Vec<DirEntry>) -> HashMap<String, i32> {
    //println!("Starting pipelineparalelism...");
    let (sender, receiver) = channel();

    let read = ReadActor {sender: sender.clone(),};
    let count = CountActor {sender: sender.clone(),};
    let combine = CombineActor {sender: sender.clone(),};

    let receiver = Arc::new(Mutex::new(receiver));

    let receiverclone = receiver.clone();

    thread::spawn(move || read.read(paths));

    thread::spawn(move || {
        println!("CountActor before receiving message"); // Debug print
        let words = match receiverclone.lock().unwrap().recv() {
            Ok(Message::FileContents(contents)) => contents,
            _ => {
                println!("Error: Failed to receive FileContents message");
                return;
            },
        };
        println!("CountActor after receiving message"); // Debug print
        count.count(words);
    });

    let receiverclone = receiver.clone();
    thread::spawn(move || {
        println!("CombineActor before receiving message"); // Debug print
        let count = match receiverclone.lock().unwrap().recv() {
            Ok(Message::WordCounts(counts)) => counts,
            _ => {
                println!("Error: Failed to receive WordCounts message");
                return;
            },
        };
        println!("CombineActor after receiving message"); // Debug print
        combine.combine(count);
    });

    // Receive the final combined word count
    let x = match receiver.lock().unwrap().recv() {
        Ok(Message::CombinedCount(combined_word_count)) => combined_word_count,
        _ => HashMap::new(),
    };
    x
}

fn main() {
    //Sequential
    let paths = fs::read_dir("books")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let now = Instant::now();
    let seq_map = squential(paths);
    let elapsed_time = now.elapsed();
    println!("Running sequential() took {} ms", elapsed_time.as_millis());
    println!("Size of seq_map: {}", seq_map.len());

    //Parallelism
    let paths = fs::read_dir("books")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let now = Instant::now();
    let par_map = parallelism(paths);
    let elapsed_time = now.elapsed();
    println!("Running parallelism() took {} ms", elapsed_time.as_millis());
    println!("Size of par_map: {}", par_map.len());

    //Pipeline parallelism
    let paths = fs::read_dir("books")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let now = Instant::now();
    let pipe_map = pipelineparalelism(paths);
    let elapsed_time: std::time::Duration = now.elapsed();
    println!(
        "Running pipeparallelism() took {} ms",
        elapsed_time.as_millis()
    );
    println!("Size of pipe_map: {}", pipe_map.len());
}
