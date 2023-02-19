use crate::error::Result;
use crate::ThreadPool;
use std::thread;

pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(_thread: u32) -> Result<Self> 
    where Self: Sized {
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
    }
}