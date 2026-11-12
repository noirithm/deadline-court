mod analysis;
mod model;

use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let json_mode = args.iter().any(|a| a == "--json");
    let path = match args.iter().find(|a| !a.starts_with("--")) {
        Some(p) => p.clone(),
        None => {
            eprintln!("usage: deadline-court [--json] <tasks.json>");
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

    let verdicts = analysis::analyze_all(&tasks);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&verdicts).unwrap());
        return;
    }
    for v in &verdicts {
        match v.slack_us {
            Some(s) => println!("{:<12} HOLDS     slack {:.1} us", v.name, s),
            None => println!("{:<12} VIOLATED", v.name),
        }
    }
}
