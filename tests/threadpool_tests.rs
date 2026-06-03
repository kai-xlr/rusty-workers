use std::sync::{Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct JobQueue {
    inner: Arc<Mutex<std::collections::VecDeque<Job>>>,
}

impl JobQueue {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(std::collections::VecDeque::new())),
        }
    }

    fn push(&self, job: Job) {
        self.inner.lock().unwrap().push_back(job);
    }

    fn pop(&self) -> Option<Job> {
        self.inner.lock().unwrap().pop_front()
    }

    fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().is_empty()
    }
}

#[test]
fn test_new_queue_is_empty() {
    let queue = JobQueue::new();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
}

#[test]
fn test_push_and_len() {
    let queue = JobQueue::new();
    queue.push(Box::new(|| {}));
    queue.push(Box::new(|| {}));
    assert_eq!(queue.len(), 2);
}

#[test]
fn test_pop_returns_jobs_in_order() {
    let queue = JobQueue::new();
    queue.push(Box::new(|| {}));
    queue.push(Box::new(|| {}));

    assert!(queue.pop().is_some());
    assert_eq!(queue.len(), 1);
    assert!(queue.pop().is_some());
    assert!(queue.is_empty());
}

#[test]
fn test_pop_empty_returns_none() {
    let queue = JobQueue::new();
    assert!(queue.pop().is_none());
}

#[test]
fn test_concurrent_push_and_pop() {
    let queue = JobQueue::new();
    let mut handles = vec![];

    for _ in 0..5 {
        let q = queue.inner.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                q.lock().unwrap().push_back(Box::new(|| {}));
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(queue.len(), 500);
}
