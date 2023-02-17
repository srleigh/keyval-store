use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc};

/// Sharable state across all thread of web server
pub struct AppState{
    read_count: AtomicU64,
    write_count: AtomicU64,
    pub startup_datetime: DateTime<Utc>,
}

impl AppState{
    pub fn new() -> AppState {
        let read_count= AtomicU64::new(0);
        let write_count= AtomicU64::new(0);
        let startup_datetime = Utc::now();
        AppState{read_count, write_count, startup_datetime}
    }

    pub fn get_reads(&self) -> u64 {
        self.read_count.load(Ordering::SeqCst)
    }

    pub fn get_writes(&self) -> u64 {
        self.write_count.load(Ordering::SeqCst)
    }

    pub fn increment_reads(&self) {
        self.read_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_writes(&self) {
        self.write_count.fetch_add(1, Ordering::SeqCst);
    }
}

