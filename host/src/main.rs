mod analysis;
mod model;
mod trace;

use std::{collections::BTreeMap, env, fs, process};

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        process::exit(1);
    })
}

fn run_trace(args: &[String]) {
    let path = args.first().unwrap_or_else(|| {
        eprintln!("usage: deadline-court trace <traceprinter-output.txt>");
        process::exit(2);
    });
    let parsed = trace::parse(&read_file(path));
    println!("events: {}", parsed.events.len());
    match parsed.cycles_per_sec {
        Some(c) => println!("cycles/sec: {c}"),
        None => println!("cycles/sec: unknown"),
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for e in &parsed.events {
        *counts.entry(e.name.as_str()).or_default() += 1;
    }
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    for (name, n) in sorted.iter().take(12) {
        println!("{:<20} {}", name, n);
    }
    println!("named threads: {}", parsed.thread_names.len());
}

fn run_tasks(args: &[String]) {
    let json_mode = args.iter().any(|a| a == "--json");
    let path = args.iter().find(|a| !a.starts_with("--")).unwrap_or_else(|| {
        eprintln!("usage: deadline-court [--json] <tasks.json>");
        process::exit(2);
    });
    let tasks = model::load_tasks(&read_file(path)).unwrap_or_else(|e| {
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

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(|a| a == "trace").unwrap_or(false) {
        run_trace(&args[1..]);
    } else {
        run_tasks(&args);
    }
}
