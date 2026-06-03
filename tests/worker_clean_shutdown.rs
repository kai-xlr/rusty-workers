use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
    mpsc,
};

use worker_pool_v1::{Job, Worker};

#[test]
fn worker_can_execute_job_and_shutdown_cleanly() {
    let (sender, receiver) = mpsc::channel::<Job>();

    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&flag);

    let receiver = Arc::new(std::sync::Mutex::new(receiver));
    let mut worker = Worker::new(0, receiver);

    sender
        .send(Box::new(move || {
            flag_clone.store(true, Ordering::SeqCst);
        }))
        .unwrap();

    drop(sender);

    worker
        .thread
        .take()
        .unwrap()
        .join()
        .unwrap();

    assert!(flag.load(Ordering::SeqCst));
    assert_eq!(worker.id, 0);
}
