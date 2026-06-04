use std::sync::{Arc, Mutex, mpsc};
use worker_pool_v1::{Job, PerformanceMetrics, Worker};

#[test]
fn multiple_workers_share_single_receiver_pipeline() {
    let results = Arc::new(Mutex::new(Vec::new()));

    let (sender, receiver) = mpsc::channel::<Job>();
    let receiver = Arc::new(Mutex::new(receiver));
    let metrics = Arc::new(PerformanceMetrics::new());

    let mut workers = Vec::new();

    for id in 0..3 {
        workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&metrics)));
    }

    for value in [10usize, 20, 30] {
        let results_clone = Arc::clone(&results);

        sender
            .send(Box::new(move || {
                let mut guard = results_clone.lock().unwrap();
                guard.push(value);
            }))
            .unwrap();
    }

    drop(sender);

    for worker in &mut workers {
        worker.join();
    }

    let mut actual = results.lock().unwrap().clone();
    actual.sort();

    assert_eq!(actual, vec![10, 20, 30]);
}
