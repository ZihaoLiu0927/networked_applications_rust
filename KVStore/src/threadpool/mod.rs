use crate::Result;


pub trait ThreadPool {
    fn new(threads: u32) -> Result<Self>
    where Self: Sized;

    fn spawn<F>(&self, job: F) 
    where F: FnOnce() + Send + 'static;
}

mod rayon;
mod naive;

