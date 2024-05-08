use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::time::Instant;
extern crate rayon;
use rayon::prelude::*;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

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
}

//Defines an actor that reads file contents and sends them as a message
struct ReadActor {
    sender: Sender<Message>,
}


impl ReadActor {
    //Function that reads contents of path
    fn read(&self, paths: Vec<DirEntry>) {
        let contents: Vec<String> = paths
            //uses parallel iterators
            .into_par_iter()
            //reads contents of files and returns them as strings
            .map(|entry| fs::read_to_string(entry.path()).unwrap_or_default())
            .collect();
        //sends a message of type FileContents
        self.sender.send(Message::FileContents(contents.clone())).unwrap();
    }
}

//Defines an actor that counts the file contents and sends them as a message
struct CountActor {
    sender: Sender<Message>,
}

impl CountActor {
    //function that counts the number of times a string has been seen in contents
    fn count(&self, words: Vec<String>) {
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
        //sends a message of type WordCount
        self.sender.send(Message::WordCounts(wordcount)).unwrap();
    }
}

//Defines an actor that combines the contents of multiple maps and sends them as a message
struct CombineActor {
    sender: Sender<Message>,
}

impl CombineActor {
    //function that combines all HashMaps into One 
    fn combine(&self, hashs: Vec<HashMap<String, i32>>) {
        let combined: HashMap<String, i32> =
            //folds all Hashs into one accumulator
            hashs.into_iter().fold(HashMap::new(), |mut acc, map| {
                //loops through all contents of map
                for (word, count) in map {
                    //accumulates all hashs into one
                    *acc.entry(word).or_insert(1) += count;
                }
                acc
            });
            //sends a message of type CombinedCount
        self.sender.send(Message::CombinedCount(combined)).unwrap();
    }
}

fn pipelineparalelism(paths: Vec<DirEntry>) -> HashMap<String, i32> {
    //Creates a communication between threads linking sender and receiver
    let (sender, receiver) = channel();

    //initializes all actors
    let read = ReadActor {sender: sender.clone(),};
    let count = CountActor {sender: sender.clone(),};
    let combine = CombineActor {sender: sender.clone(),};

    //Creates a cloneable receiver
    let receiver = Arc::new(Mutex::new(receiver));

    //calls read function of ReadActor
    read.read(paths);

    //checks reeiver of it has a message of type FileContents with data filecontents if so set words to filecontents
    let words = match receiver.lock().unwrap().recv() {
        Ok(Message::FileContents(filecontents)) => filecontents,
        _ =>  return HashMap::new(),
    };

    //calls count function of CountActor
    count.count(words);

    //checks reeiver of it has a message of type WordCounts with data count if so set count to count
    let count = match receiver.lock().unwrap().recv() {
        Ok(Message::WordCounts(count)) => count,
        _ => return HashMap::new(),
    };

    //calls combine function of CombineActor
    combine.combine(count);

   //checks reeiver of it has a message of type CombinedCount with data combined_count if so return combined_count
    return match receiver.lock().unwrap().recv() {
        Ok(Message::CombinedCount(combined_count)) => combined_count,
        _ => HashMap::new(),
    };

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
