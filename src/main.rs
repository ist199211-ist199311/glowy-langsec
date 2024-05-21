use std::{env, fs, io};

use glowy::analyze_files;

fn main() {
    let path = env::args().nth(1);

    let mut files: Vec<(String, String)> = Vec::new();

    if let Some(path) = path {
        let content = fs::read_to_string(&path).expect("Failed to read file at specified path");
        files.push((path, content));
    } else {
        let content = io::read_to_string(io::stdin()).expect("Failed to read input from stdin");
        files.push(("<stdin>".to_string(), content));
    };

    let files = files
        .iter()
        .map(|(file_name, contents)| (file_name.as_str(), contents.as_str()));

    let context = analyze_files(files);
    dbg!(context);
}
