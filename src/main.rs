use std::time::Duration;
use worker_pool_v1::ThreadPool;

fn main() {
    println!("Initializing Worker Pool V1 (4 Workers)...");

    let mut pool = ThreadPool::new(4);

    println!("Dispatching 20 background compute jobs...");

    for i in 0..20 {
        pool.execute(move || {
            println!(
                "   -> Job {} starting on Thread {:?}",
                i,
                std::thread::current().id()
            );

            std::thread::sleep(Duration::from_millis(15));

            println!("   <- Job {} completed", i);
        });
    }

    println!("Initiating Explicit Graceful Shutdown. Waiting for workers to drain pipeline...");

    pool.shutdown();

    println!("All threads successfully joined. Process exiting cleanly.");
}
