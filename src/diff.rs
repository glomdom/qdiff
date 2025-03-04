use memmap2::Mmap;

use crate::{patch::PatchFile, rolling_xxhash::RollingXXHash};
use std::{
    collections::{HashMap, HashSet},
    fs::{File, metadata},
};

#[derive(Debug)]
pub struct FileDiff {
    pub added: Vec<(usize, Vec<u8>)>, // (offset, data)
    pub removed: Vec<(usize, usize)>, // (offset, size)
}

impl FileDiff {
    pub fn new() -> Self {
        Self {
            added: Vec::new(),
            removed: Vec::new(),
        }
    }
}

pub fn compute_diff(file_a: &str, file_b: &str, window_size: usize) -> std::io::Result<FileDiff> {
    let window_size = if window_size == 0 {
        auto_detect_window_size(file_a)
    } else {
        window_size
    };

    let mut diff = FileDiff::new();

    let file_a = File::open(file_a)?;
    let file_b = File::open(file_b)?;

    let mmap_a = unsafe { Mmap::map(&file_a)? };
    let mmap_b = unsafe { Mmap::map(&file_b)? };

    let data_a = &mmap_a[..];
    let data_b = &mmap_b[..];

    // for byte by byte processing
    if data_a.len() <= window_size && data_b.len() <= window_size {
        for (i, (&byte_a, &byte_b)) in data_a.iter().zip(data_b.iter()).enumerate() {
            if byte_a != byte_b {
                diff.added.push((i, vec![byte_b]));
                diff.removed.push((i, 1));
            }
        }

        return Ok(diff);
    }

    let mut hash_map_a: HashMap<u64, usize> = HashMap::new(); // hash -> offset
    let mut hash_set_b: HashSet<u64> = HashSet::new();

    let mut rolling_a = RollingXXHash::new(window_size, 0);
    let mut rolling_b = RollingXXHash::new(window_size, 0);

    for (pos, window) in data_a.windows(window_size).enumerate() {
        rolling_a.add_bytes(window);
        hash_map_a.insert(rolling_a.current_hash(), pos);
    }

    for (pos, window) in data_b.windows(window_size).enumerate() {
        rolling_b.add_bytes(window);

        let hash_b = rolling_b.current_hash();
        hash_set_b.insert(hash_b);

        if !hash_map_a.contains_key(&hash_b) {
            diff.added.push((pos, window.to_vec()));
        }
    }

    for (&hash_a, &pos) in &hash_map_a {
        if !hash_set_b.contains(&hash_a) {
            diff.removed.push((pos, window_size));
        }
    }

    Ok(diff)
}

pub fn auto_detect_window_size(filename: &str) -> usize {
    let file_size = metadata(filename).map(|m| m.len()).unwrap_or(1024);
    let window_size = file_size.min((file_size / 10000).max(32).min(512)) as usize;

    println!(
        "+ autodetected window size of {} bytes for file size of {} bytes",
        window_size, file_size
    );

    window_size
}

pub fn compute_diff_and_save(
    file_a: &str,
    file_b: &str,
    patch_file: &str,
    window_size: usize,
) -> std::io::Result<()> {
    let diff = compute_diff(file_a, file_b, window_size)?;
    let patch = PatchFile {
        added: diff.added,
        removed: diff.removed,
    };

    patch.save_to_file(patch_file)?;
    println!("saved patch to {}", patch_file);

    Ok(())
}
