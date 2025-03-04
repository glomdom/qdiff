mod diff;
mod patch;
mod rolling_xxhash;

use diff::compute_diff_and_save;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage:");
        eprintln!(
            "  {} diff <file1> <file2> <patch.qdf> <window size (0 for auto)>",
            args[0]
        );
        eprintln!("  {} patch <original> <patch.qdf> <output>", args[0]);

        std::process::exit(1);
    }

    let command = &args[1];
    if command == "diff" {
        let file1 = &args[2];
        let file2 = &args[3];
        let patch_file = &args[4];
        let window_size: usize = args[5].parse().expect("invalid window size");

        if let Err(err) = compute_diff_and_save(file1, file2, patch_file, window_size) {
            eprintln!("Error: {}", err);
        }
    }
}
