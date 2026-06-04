use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::job::Job;
use crate::metrics::PerformanceMetrics;
use crate::worker::Worker;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    metrics: Arc<PerformanceMetrics>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let metrics = Arc::new(PerformanceMetrics::new());

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&metrics)));
        }

        Self {
            workers,
            sender: Some(sender),
            metrics,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Box::new(f);

        self.metrics.increment_submitted();
        self.sender.as_ref().unwrap().send(job).unwrap();
    }

    pub fn jobs_completed(&self) -> usize {
        self.metrics.jobs_completed()
    }

    pub fn shutdown(&mut self) {
        if self.sender.is_some() {
            drop(self.sender.take());

            for worker in &mut self.workers {
                worker.join();
            }

            println!("\n==============================================");
            println!("      WORKER POOL V1 SHUTDOWN DIAGNOSTICS     ");
            println!("==============================================");
            println!("Workers Configured    : {}", self.workers.len());
            println!("Total Jobs Enqueued   : {}", self.metrics.jobs_submitted());
            println!("Total Jobs Completed  : {}", self.metrics.jobs_completed());
            println!("Pending Tasks Abandoned: {}", self.metrics.pending_jobs());
            println!(
                "Average Job Runtime   : {:.2} ms",
                self.metrics.average_runtime_ms()
            );
            println!("==============================================\n");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if self.sender.is_some() {
            drop(self.sender.take());

            for worker in &mut self.workers {
                worker.join();
            }
        }
    }
}
