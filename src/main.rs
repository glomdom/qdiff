mod diff;
mod rolling_xxhash;

use diff::compute_diff;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} diff <file1> <file2> <window_size>", args[0]);
        std::process::exit(1);
    }

    let file1 = &args[2];
    let file2 = &args[3];
    let window_size: usize = args[4].parse().expect("Invalid window size");

    match compute_diff(file1, file2, window_size) {
        Ok(diff) => {
            println!("+ differences Found:");
            println!("+ added: {:?}", diff.added);
            println!("+ removed: {:?}", diff.removed);
        }
        Err(err) => eprintln!("Error: {}", err),
    }
}
