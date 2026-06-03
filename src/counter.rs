use std::sync::{Arc, Mutex};
use std::thread;

pub fn run_counter_demo() {
    let counter = Arc::new(Mutex::new(0u64));
    let mut handles = Vec::new();

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                let mut val = counter.lock().unwrap();
                *val += 1;
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let final_val = *counter.lock().unwrap();
    println!("Shared counter final value: {} (expected: 10000)", final_val);
    assert_eq!(final_val, 10000);
}
