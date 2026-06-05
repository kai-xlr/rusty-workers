use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::job::Job;
use crate::metrics::PerformanceMetrics;

pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        _id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        metrics: Arc<PerformanceMetrics>,
    ) -> Self {
        let thread = thread::spawn(move || {
            while let Ok(job) = receiver.lock().unwrap().recv() {
                let start = std::time::Instant::now();

                job();

                let elapsed = start.elapsed().as_micros() as u64;

                metrics.increment_completed();
                metrics.add_runtime(elapsed);
            }
        });

        Self {
            thread: Some(thread),
        }
    }

    pub fn join(&mut self) {
        if let Some(thread) = self.thread.take()
            && let Err(e) = thread.join() {
                std::panic::resume_unwind(e);
            }
    }
}
