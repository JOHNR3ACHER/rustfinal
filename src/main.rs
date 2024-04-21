use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::env;
use std::fs;
use std::fs::ReadDir;

fn squential(paths:ReadDir){
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
}


fn main() {
    let paths = fs::read_dir("/workspaces/rustfinal/books").unwrap();

    squential(paths);

}
