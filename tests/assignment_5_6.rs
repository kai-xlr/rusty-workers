use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use worker_pool_v1::ThreadPool;

#[test]
fn thread_pool_executes_twenty_jobs() {
    let pool = ThreadPool::new(4);

    let results = Arc::new(Mutex::new(Vec::new()));

    for id in 0..20usize {
        let results_clone = Arc::clone(&results);

        pool.execute(move || {
            thread::sleep(Duration::from_millis(20));

            let mut guard = results_clone.lock().unwrap();
            guard.push(id);
        });
    }

    thread::sleep(Duration::from_millis(500));

    let mut completed = results.lock().unwrap().clone();

    assert_eq!(completed.len(), 20);

    completed.sort();
    assert_eq!(completed, (0usize..20).collect::<Vec<_>>());
}
