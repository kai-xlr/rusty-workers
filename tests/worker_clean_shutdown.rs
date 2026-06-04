use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
    mpsc,
};

use worker_pool_v1::{Job, PerformanceMetrics, Worker};

#[test]
fn worker_can_execute_job_and_shutdown_cleanly() {
    let (sender, receiver) = mpsc::channel::<Job>();

    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&flag);

    let receiver = Arc::new(std::sync::Mutex::new(receiver));
    let metrics = Arc::new(PerformanceMetrics::new());
    let mut worker = Worker::new(0, receiver, metrics);

    sender
        .send(Box::new(move || {
            flag_clone.store(true, Ordering::SeqCst);
        }))
        .unwrap();

    drop(sender);

    worker.join();

    assert!(flag.load(Ordering::SeqCst));
}
