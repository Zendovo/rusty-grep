use std::env;
use std::fs::File;
use std::io;
use std::io::{prelude::*, BufReader};
use std::process;

mod parser;
mod matcher;

use matcher::match_pattern;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {    
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Expected a pattern as the second argument");
        process::exit(1);
    }

    if args.get(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = args.get(2).unwrap();
    let mut input_line = String::new();

    if args.len() >= 4 {
        // Get all file names
        match_files(&args[3..], pattern);
    }
    io::stdin().read_line(&mut input_line).unwrap();

    // Trim trailing newline for correct '$' anchor matching
    let trimmed_input = input_line.trim_end_matches('\n');

    if match_pattern(trimmed_input, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

fn match_files(files: &[String], pattern: &str) {
    let mut any_match = false;

    for file_name in files {
        // Open the file and read each line
        let file = match File::open(file_name) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error opening file {}: {}", file_name, e);
                process::exit(1);
            }
        };

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let trimmed_line = line.trim_end_matches('\n');
            if match_pattern(trimmed_line, &pattern) {
                println!("{}", line);
                any_match = true;
            }
        }
    }

    if any_match {
        process::exit(0)
    } else {
        process::exit(1)
    }
}