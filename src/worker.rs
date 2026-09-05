use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    thread::{self, JoinHandle},
};

use log::trace;

use crate::pool::Job;

pub(crate) struct Worker {
    pub(crate) id: usize,
    pub(crate) thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        trace!("Thread {id} working on request...");
                        job()
                    }
                    Err(_) => {
                        trace!("Thread {id} disconnected; shutting down");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}
