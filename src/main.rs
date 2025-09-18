use std::env;
use std::fs::{File, read_dir};
use std::path::Path;
use std::io;
use std::io::{prelude::*, BufReader};
use std::process;

mod parser;
mod matcher;
mod cli;

use matcher::match_pattern;
use cli::Arguments;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    let args: Vec<String> = env::args().collect();
    let arguments = match Arguments::parse(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let mut files = Vec::new();
    if !arguments.files.is_empty() {

        // Take input from files
        if arguments.recursive {
            for target in &arguments.files {
                collect_files_recursively(Path::new(target), &mut files);
            }
        } else {
            files = arguments.files.clone();
        }
        match_files(&files, &arguments.pattern);
    } else {

        // Take input from stdin
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let trimmed_input = input_line.trim_end_matches('\n');

        if match_pattern(trimmed_input, &arguments.pattern) {
            process::exit(0)
        } else {
            process::exit(1)
        }
    }
}
fn collect_files_recursively(path: &Path, files: &mut Vec<String>) {
    if path.is_file() {
        files.push(path.to_string_lossy().to_string());
    } else if path.is_dir() {
        if let Ok(entries) = read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    collect_files_recursively(&entry_path, files);
                }
            }
        }
    }
}

fn match_files(files: &[String], pattern: &str) {
    let mut any_match = false;
    let multiple_files = files.len() > 1;

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
                if multiple_files {
                    print!("{}:", file_name);
                }
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