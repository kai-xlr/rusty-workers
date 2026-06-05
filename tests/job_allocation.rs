use std::sync::{Arc, Mutex};
use worker_pool_v1::Job;

#[test]
fn job_can_be_allocated_and_executed() {
    let executed_flag = Arc::new(Mutex::new(false));

    let flag_clone = Arc::clone(&executed_flag);

    let job: Job = Box::new(move || {
        let mut guard = flag_clone.lock().unwrap();
        *guard = true;
    });

    job();

    let final_state = *executed_flag.lock().unwrap();
    assert!(
        final_state,
        "The job did not mutate the shared state successfully."
    );
}
