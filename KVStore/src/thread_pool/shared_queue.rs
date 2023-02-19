use crate::Result;
use crate::ThreadPool;

pub struct SharedQueueThreadPool {

}


impl ThreadPool for SharedQueueThreadPool {
    fn new(_thread: u32) -> Result<Self> {
        todo!()
    }

    fn spawn<F>(&self, job: F) 
    where F: FnOnce() + Send + 'static 
    {
        todo!()
    }
}

    