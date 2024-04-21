use std::collections::HashMap;
use std::fs::DirEntry;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::env;
use std::fs;
use std::fs::ReadDir;

fn squential(paths:ReadDir){
    let mut wordcount: HashMap<&str,i32> = HashMap::new();
    for path in paths {
        let contents = fs::read_to_string(path.unwrap().path()).expect("Should have been able to read the file");
        let words:Vec<&str> = contents.split(&[' ','\n','.','!','?']).collect();
        for word in words{
            if !wordcount.contains_key(word){
                wordcount.insert(word, 0);
            }else{
                *wordcount.get_mut(word).unwrap() += 1;
            }
        }
    }  
}


fn main() {
    let paths = fs::read_dir("/workspaces/rustfinal/books").unwrap();

    squential(paths);

}
