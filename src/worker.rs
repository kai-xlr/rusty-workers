use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::job::Job;

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = {
                    let receiver = receiver.lock().unwrap();
                    receiver.recv()
                };
                match message {
                    Ok(job) => {
                        job();
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
