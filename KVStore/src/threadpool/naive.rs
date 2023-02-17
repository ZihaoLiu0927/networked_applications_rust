use crate::error::Result;
use crate::ThreadPool;
use std::thread::JoinHandle;
use std::{sync::mpsc, thread};

pub struct NaiveThreadPool {
    // workers: Vec<Worker>,
}


type Job = Box<dyn FnOnce() + Send + 'static>;
// struct Worker {
//     thread: JoinHandle<()>,
// }

// impl Worker {
//     pub fn new(thread: JoinHandle<()>) -> Self{
//         Self {
//             thread,
//         }
//     }
// }

impl ThreadPool for NaiveThreadPool {
    fn new(size: u32) -> Result<Self> 
    where Self: Sized {
        // let mut workers: Vec<Worker> = Vec::new();
        // Ok(
        //     Self {
        //         workers: workers,
        //     }
        // )
        Ok(
            Self{}
        )
    }

    fn spawn<F>(&self, job: F) 
    where F: FnOnce() + Send + 'static 
    {
        thread::spawn(|| {
            job()
        });
        //Worker::new(handler);
    }
}