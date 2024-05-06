use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::time::Instant;
extern crate rayon;
use rayon::prelude::*;

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

fn read(paths: Vec<DirEntry>) -> Vec<String> {
    paths
        //uses parallel iterators
        .into_par_iter()
        .map(|entry| fs::read_to_string(entry.path()).unwrap_or_default())
        .collect()
}

fn count(words: Vec<String>) -> Vec<HashMap<String, i32>> {
    words
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
        .collect()
}

fn combine(hashs: Vec<HashMap<String, i32>>) -> HashMap<String, i32> {
    hashs.into_iter().fold(HashMap::new(), |mut acc, map| {
        //loops through all contents of map
        for (word, count) in map {
            //accumulates all hashs into one
            *acc.entry(word).or_insert(1) += count;
        }
        acc
    })
}

fn main() {
    //Sequential
    let paths = fs::read_dir("books").unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    let now = Instant::now();
    let seq_map = squential(paths);
    let elapsed_time = now.elapsed();
    println!("Running sequential() took {} ms", elapsed_time.as_millis());
    println!("Size of seq_map: {}", seq_map.len());

    //Parallelism
    let paths = fs::read_dir("books").unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    let now = Instant::now();
    let par_map = parallelism(paths);
    let elapsed_time = now.elapsed();
    println!("Running parallelism() took {} ms", elapsed_time.as_millis());
    println!("Size of par_map: {}", par_map.len());

    //Pipeline parallelism
    let paths = fs::read_dir("books").unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    let now = Instant::now();
    let words = read(paths);
    let count = count(words);
    let pipe_map = combine(count);
    let elapsed_time: std::time::Duration = now.elapsed();
    println!("Running pipeparallelism() took {} ms",elapsed_time.as_millis());
    println!("Size of pipe_map: {}", pipe_map.len());
}
