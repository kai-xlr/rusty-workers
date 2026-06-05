use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use worker_pool_v1::Job;

#[test]
fn single_worker_thread_executes_sequential_jobs() {
    let results = Arc::new(Mutex::new(Vec::new()));
    let (sender, receiver) = mpsc::channel::<Job>();

    let handle = thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    for &val in &[10, 20, 30] {
        let results_clone = Arc::clone(&results);
        sender
            .send(Box::new(move || {
                let mut guard = results_clone.lock().unwrap();
                guard.push(val);
            }))
            .unwrap();
    }
    drop(sender);
    handle.join().unwrap();
    let final_results = results.lock().unwrap();
    assert_eq!(*final_results, vec![10, 20, 30]);
}
