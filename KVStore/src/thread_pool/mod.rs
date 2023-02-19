use crate::Result;


pub trait ThreadPool {
    fn new(threads: u32) -> Result<Self>
    where Self: Sized;

    fn spawn<F>(&self, job: F) 
    where F: FnOnce() + Send + 'static;
}

mod rayon;
mod naive;
mod shared_queue;

pub use self::naive::NaiveThreadPool;
pub use self::rayon::RayonThreadPool;
pub use self::shared_queue::SharedQueueThreadPool;

