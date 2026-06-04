use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct PerformanceMetrics {
    jobs_submitted: AtomicUsize,
    jobs_completed: AtomicUsize,
    total_runtime_micros: AtomicU64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            jobs_submitted: AtomicUsize::new(0),
            jobs_completed: AtomicUsize::new(0),
            total_runtime_micros: AtomicU64::new(0),
        }
    }

    pub fn increment_submitted(&self) {
        self.jobs_submitted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_completed(&self) {
        self.jobs_completed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_runtime(&self, micros: u64) {
        self.total_runtime_micros.fetch_add(micros, Ordering::Relaxed);
    }

    pub fn jobs_submitted(&self) -> usize {
        self.jobs_submitted.load(Ordering::Relaxed)
    }

    pub fn jobs_completed(&self) -> usize {
        self.jobs_completed.load(Ordering::Relaxed)
    }

    pub fn pending_jobs(&self) -> usize {
        let submitted = self.jobs_submitted.load(Ordering::Relaxed);
        let completed = self.jobs_completed.load(Ordering::Relaxed);
        if submitted > completed {
            submitted - completed
        } else {
            0
        }
    }

    pub fn average_runtime_ms(&self) -> f64 {
        let completed = self.jobs_completed.load(Ordering::Relaxed);
        if completed == 0 {
            return 0.0;
        }
        let total_micros = self.total_runtime_micros.load(Ordering::Relaxed);
        (total_micros as f64 / 1000.0) / completed as f64
    }
}
