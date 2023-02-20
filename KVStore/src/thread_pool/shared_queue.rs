use crate::{
    ThreadPool,
    error::Result
};

use std::{
    panic::{self, AssertUnwindSafe}, 
    thread::{self, JoinHandle},
    sync::{Arc, Mutex},
    sync::mpsc::{self, Sender, Receiver},
};


pub struct SharedQueueThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[allow(dead_code)]
struct Worker {
    id: u32,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: u32, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move|| loop {
            let message = receiver
                .lock()
                .expect("unable to acquire lock.")
                .recv();

            match message {
                Ok(job) => {
                    if let Err(e) = panic::catch_unwind(AssertUnwindSafe(job)) {
                        eprintln!("{:?} executed a job with panic", e);
                    }
                },

                Err(_sender_shutdown) => {
                    break;
                }
            }
        });

        Self{
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(thread: u32) -> Result<Self> 
    where Self: Sized 
    {
        let (sender, receiver): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        // need to add error handling
        let mut workers = Vec::with_capacity(thread.try_into().unwrap());

        for id in 0..thread {
            workers.push(
                Worker::new(id, Arc::clone(&receiver))
            );
        }

        Ok(
            Self {
                sender: Some(sender),
                workers,
            }
        )
    }

    fn spawn<F>(&self, job: F) 
    where F: FnOnce() + Send + 'static
    {
        let job = Box::new(job);
        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .unwrap();
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) { 

        drop(self.sender.take());

        for worker in &mut self.workers {
            
            if let Some(worker) = worker.thread.take() {
                worker.join().expect("Error when waiting for worker finishing its job.");
            }
        }

    }
}