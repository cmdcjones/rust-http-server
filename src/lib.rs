use core::fmt;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

pub struct PoolCreationError;

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error creating thread pool!")
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

/// Create a new ThreadPool.
///
/// The size is the number of threads in the pool.
///
/// # Panics
///
/// The `new` function will panic if the size is zero.
impl ThreadPool {
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size > 0 {
            let (sender, receiver) = mpsc::channel();

            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            return Ok(ThreadPool {
                workers,
                sender: Some(sender),
            });
        }
        return Err(PoolCreationError);
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            worker.thread.take().unwrap().join().unwrap();
        }
    }
}

struct Worker {
    _id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let id = id + 1;
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                    println!("Worker {id} finished a job.");
                }
                Err(_) => {
                    break;
                }
            }
        });
        Self {
            _id: id,
            thread: Some(thread),
        }
    }
}
