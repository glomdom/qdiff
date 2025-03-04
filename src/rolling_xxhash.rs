use std::collections::VecDeque;

use xxhash_rust::xxh64::xxh64;

pub struct RollingXXHash {
    window: VecDeque<u8>, // bytes window
    window_size: usize,   // size of the window
    current_hash: u64,    // current xxhash value
    seed: u64,            // seed for hashing
}

impl RollingXXHash {
    pub fn new(window_size: usize, seed: u64) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            current_hash: 0,
            seed,
        }
    }

    pub fn add_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.add_byte(byte);
        }
    }

    pub fn add_byte(&mut self, byte: u8) {
        if self.window.len() == self.window_size {
            self.remove_byte();
        }

        self.window.push_back(byte);
        self.current_hash ^= xxh64(&[byte], self.seed);
    }

    fn remove_byte(&mut self) {
        if let Some(old_byte) = self.window.pop_front() {
            self.current_hash ^= xxh64(&[old_byte], self.seed);
        }
    }

    pub fn current_hash(&self) -> u64 {
        self.current_hash
    }
}
