mod analysis;
mod model;

use std::{env, fs, process};

fn main() {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: deadline-court <tasks.json>");
            process::exit(2);
        }
    };
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        process::exit(1);
    });
    let tasks = model::load_tasks(&text).unwrap_or_else(|e| {
        eprintln!("bad task file: {e}");
        process::exit(1);
    });

    for (i, t) in tasks.iter().enumerate() {
        match analysis::slack(&tasks, i) {
            Some(s) => println!("{:<12} HOLDS     slack {:.1} us", t.name, s),
            None => println!("{:<12} VIOLATED", t.name),
        }
    }
}
