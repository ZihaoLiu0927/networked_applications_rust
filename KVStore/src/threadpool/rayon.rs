use std::{sync::mpsc, thread};
use std::thread::JoinHandle;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use std::sync::Mutex;
use crate::error::Result;
use crate::ThreadPool;

pub struct RayonThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

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
                Ok(job) => job(),
                Err(_) => {
                    println!("worker exits.");
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

impl ThreadPool for RayonThreadPool {
    fn new(size: u32) -> Result<Self> 
    where Self: Sized 
    {
        let (tx, rx): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        // need to add error handling
        let mut workers = Vec::with_capacity(size.try_into().unwrap());

        for id in 0..size {
            workers.push(
                Worker::new(id, Arc::clone(&rx))
            );
        }

        Ok(
            Self {
                sender: Some(tx),
                workers,
            }
        )
    }

    fn spawn<F>(&self, job: F) 
    where F: FnOnce() + Send + 'static
    {
        let job = Box::new(job);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for RayonThreadPool {
    fn drop(&mut self) { 

        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            if let Some(worker) = worker.thread.take() {
                worker.join().expect("Error when waiting for worker finishing its job.");
            }
        }
    }
}