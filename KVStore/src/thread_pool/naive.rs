use std::thread;

use crate::{error::Result, ThreadPool};

pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(_thread: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(|| job());
    }
}
