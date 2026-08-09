//! Dependency-free progress reporting for long offline compiler operations.

use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

const BAR_WIDTH: u64 = 24;

pub struct Progress {
    label: &'static str,
    total: u64,
    started: Instant,
    last_update: Instant,
    terminal: bool,
    finished: bool,
}

impl Progress {
    pub fn new(label: &'static str, total: usize) -> Self {
        let now = Instant::now();
        let mut progress = Self {
            label,
            total: total.max(1) as u64,
            started: now,
            last_update: now.checked_sub(Duration::from_secs(2)).unwrap_or(now),
            terminal: io::stderr().is_terminal(),
            finished: false,
        };
        progress.set(0);
        progress
    }

    pub fn set(&mut self, current: usize) {
        let current = (current as u64).min(self.total);
        let now = Instant::now();
        if current != self.total && now.duration_since(self.last_update) < Duration::from_secs(1) {
            return;
        }
        self.last_update = now;
        let percent = current.saturating_mul(100) / self.total;
        let elapsed = now.duration_since(self.started).as_secs();
        if self.terminal {
            let filled = current.saturating_mul(BAR_WIDTH) / self.total;
            eprint!("\r{} [", self.label);
            for index in 0..BAR_WIDTH {
                eprint!("{}", if index < filled { '=' } else { ' ' });
            }
            eprint!("] {current}/{} {percent:>3}% {elapsed}s", self.total);
            let _ = io::stderr().flush();
        } else {
            eprintln!(
                "progress: {} {current}/{} ({percent}%, {elapsed}s)",
                self.label, self.total
            );
        }
        if current == self.total {
            self.finished = true;
            if self.terminal {
                eprintln!();
            }
        }
    }

    pub fn finish(&mut self) {
        self.set(self.total as usize);
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        if !self.finished && self.terminal {
            eprintln!();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_file(
    path: impl AsRef<Path>,
    label: &'static str,
) -> Result<Vec<u8>, crate::SourceUnavailable> {
    let path = path.as_ref();
    let total = usize::try_from(std::fs::metadata(path)?.len()).unwrap_or(usize::MAX);
    let mut progress = Progress::new(label, total);
    let mut file = std::fs::File::open(path)?;
    let mut bytes = Vec::with_capacity(total.min(512 * 1024 * 1024));
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
        progress.set(bytes.len());
    }
    progress.finish();
    Ok(bytes)
}
