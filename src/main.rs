use std::collections::HashMap;
use std::fs::DirEntry;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::env;
use std::fs;
use std::fs::ReadDir;

fn squential(paths:ReadDir){
    for path in paths {
        let contents = fs::read_to_string(path.unwrap().path()).expect("Should have been able to read the file");
        println!("{} + 1", contents);
    }
    
}


fn main() {
    let paths = fs::read_dir("/workspaces/rustfinal/books").unwrap();

    squential(paths);

}
