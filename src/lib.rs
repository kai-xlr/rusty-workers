pub mod job;
pub mod metrics;
pub mod threadpool;
pub mod worker;

pub use job::Job;
pub use metrics::PerformanceMetrics;
pub use threadpool::ThreadPool;
pub use worker::Worker;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_enqueue_success() {
        let pool = ThreadPool::new(2);
        pool.execute(|| {});
    }

    #[test]
    fn test_single_job_execution() {
        let mut pool = ThreadPool::new(1);
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&flag);

        pool.execute(move || {
            flag_clone.store(true, Ordering::SeqCst);
        });

        pool.shutdown();

        assert!(flag.load(Ordering::SeqCst), "The background job failed to execute and set the flag.");
    }

    #[test]
    fn test_high_volume_stress() {
        let mut pool = ThreadPool::new(4);
        let counter = Arc::new(Mutex::new(0));
        let total_jobs = 100;

        for _ in 0..total_jobs {
            let counter_clone = Arc::clone(&counter);
            pool.execute(move || {
                let mut lock = counter_clone.lock().unwrap();
                *lock += 1;
            });
        }

        pool.shutdown();

        assert_eq!(*counter.lock().unwrap(), total_jobs, "High volume task aggregation mismatch.");
        assert_eq!(pool.jobs_completed(), total_jobs, "Telemetry metrics failed to match executed task counts.");
    }

    #[test]
    fn test_clean_shutdown() {
        let start = std::time::Instant::now();
        let mut pool = ThreadPool::new(2);

        pool.execute(|| {
            std::thread::sleep(Duration::from_millis(20));
        });

        pool.shutdown();
        let duration = start.elapsed();

        assert!(duration < Duration::from_millis(200), "Shutdown blocked the runtime for too long.");
    }

    #[test]
    #[should_panic(expected = "Intentional System Disruption")]
    fn test_job_panic_behavior() {
        let mut pool = ThreadPool::new(2);

        pool.execute(|| {
            panic!("Intentional System Disruption");
        });

        std::thread::sleep(Duration::from_millis(20));

        pool.shutdown();
    }
}
