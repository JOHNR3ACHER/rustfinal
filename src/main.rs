use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
//use std::fs::ReadDir;
use std::time::Instant;
extern crate rayon;
use rayon::prelude::*;

// use std::thread;
// use std::env;

fn squential(paths: Vec<DirEntry>) -> HashMap<String, i32> {
    let mut wordcount: HashMap<String, i32> = HashMap::new();
    for path in paths {
        let contents = fs::read_to_string(path.path()).expect("Error");
        let words: Vec<&str> = contents.split_whitespace().collect();
        for word in words {
            if !wordcount.contains_key(word) {
                wordcount.insert(word.to_string(), 1);
            } else {
                *wordcount.get_mut(word).unwrap() += 1;
            }
        }
    }
    return wordcount;
}

fn parallelism(paths: Vec<DirEntry>) -> HashMap<String, i32> {
    paths
        .into_par_iter()
        .map(|entry| {
            let contents = fs::read_to_string(entry.path()).expect("Error");
            let mut wordcount: HashMap<String, i32> = HashMap::new();
            //let words: Vec<&str> = contents.split_whitespace().collect();
            for word in contents.split_whitespace() {
                *wordcount.entry(word.to_string()).or_insert(0) += 1;
            }
            wordcount
        })
        .reduce(
            || HashMap::new(),
            |mut acc, map| {
                for (word, count) in map {
                    *acc.entry(word).or_insert(0) += count;
                }
                acc
            },
        )
}

// fn pipeparallelism(paths: Vec<DirEntry>) -> HashMap<String, i32> {
//     let mut wordcount: HashMap<String, i32> = HashMap::new();

//     return wordcount;
// }

#[allow(unused_variables)]
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

    //Parallelism
    let paths = fs::read_dir("books")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let now = Instant::now();
    let par_map = parallelism(paths);
    let elapsed_time = now.elapsed();
    println!("Running parallelism() took {} ms", elapsed_time.as_millis());

    // //Pipeline parallelism
    // let paths = fs::read_dir("books").unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    // let now = Instant::now();
    // let pipe_map = pipeparallelism(paths);
    // let elapsed_time = now.elapsed();
    // println!("Running rayon() took {} ms", elapsed_time.as_millis());
}
