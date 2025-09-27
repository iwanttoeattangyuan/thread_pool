//! A simple thread pool implementation in Rust
use std::thread;
use crossbeam_channel as mpsc;
//use std::sync::{Arc, Mutex};

/// type alias for a job that can be sent to workers
type Job = Box<dyn FnOnce() + Send + 'static>; 

/// ThreadPool struct managing a pool of worker threads

/// and a channel for sending jobs to them
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {

    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "ThreadPool size must be greater than zero");

        let (sender, receiver) = mpsc::unbounded();//channel for sending jobs to workers
        //let receiver = Arc::new(Mutex::new(receiver));//shared receiver among workers
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            
            //let receiver_clone = Arc::clone(&receiver);
            workers.push(Worker::new(id,receiver.clone()));

        }
        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Some(sender) = &self.sender {
            if let Err(err) = sender.send(job) {
                eprintln!("Failed to send job to worker: {}", err);
    }
}
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            //println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}


impl Worker {
    fn new(id: usize, receiver: mpsc::Receiver<Job>) -> Worker {
        let thread = thread::spawn(move || {

           while let Ok(job) = receiver.recv() {
               //println!("Worker {id} got a job; executing.");
               job();
           }

            //println!("Worker {id} disconnected; shutting down.");
        });

        Worker { id, thread: Some(thread)  }
    }
}

