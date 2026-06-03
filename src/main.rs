mod counter;
mod threadpool;

use std::thread;
use std::time::Duration;

use threadpool::JobQueue;

fn main() {
    println!("=== Task 1: Shared Counter ===\n");
    counter::run_counter_demo();

    println!("\n=== Task 2: Job Queue ===\n");

    let queue = JobQueue::new();

    for id in 0..10 {
        queue.push(Box::new(move || {
            println!("  Job {} says hello from {:?}", id, thread::current().id());
        }));
    }

    assert_eq!(queue.len(), 10);

    while !queue.is_empty() {
        if let Some(job) = queue.pop() {
            job();
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }

    assert!(queue.is_empty());
    println!("\nAll jobs completed successfully!");
}
