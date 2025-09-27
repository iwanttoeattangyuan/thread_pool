//! A simple thread pool implementation in Rust
use std::thread;
use crossbeam_deque::{Injector, Stealer, Worker as DequeWorker}; 
use rand::seq::SliceRandom; 
use std::sync::Arc;
/// type alias for a job that can be sent to workers
type Job = Box<dyn FnOnce() + Send + 'static>; 

/// ThreadPool struct managing a pool of worker threads

/// and a channel for sending jobs to them
pub struct ThreadPool {
    workers: Vec<Worker>,
    global_injector: Arc<Injector<Message>>,// Global injector for work stealing
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
/// ENUM distinguishing between new jobs and termination signals
enum Message {
    NewJob(Job),
    Terminate,
}
///


impl ThreadPool {

    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "ThreadPool size must be greater than zero");

        let global_injector = Arc::new(Injector::new());

        let mut workers = Vec::with_capacity(size);
        let mut local_queues:Vec<DequeWorker<Message>>  = (0..size).map(|_| DequeWorker::new_fifo()).collect();
        let stealers: Vec<Stealer<Message>> = local_queues.iter().map(|w| w.stealer()).collect();
        for id in 0..size {
            let local_queue = local_queues.remove(0);
            let injector_clone = Arc::clone(&global_injector); 
            let stealers_clone = stealers.clone();
            
            workers.push(Worker::new(id, move || {
                worker_loop(local_queue, injector_clone, stealers_clone);
            }));
        }

        ThreadPool { workers, global_injector }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.global_injector.push(Message::NewJob(job));
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 发送关闭信号给所有工作线程
       for _ in 0..self.workers.len() {
            self.global_injector.push(Message::Terminate);
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}


impl Worker {
    /// receives a closure representing thread's logic
    fn new<F>(id: usize, logic: F) -> Worker
    where
        F: FnOnce() + Send + 'static,
    {
        let thread = thread::spawn(logic);

        Worker { id, thread: Some(thread) }
    }
}

fn worker_loop(
    local_queue: DequeWorker<Message>,
    global_injector: Arc<Injector<Message>>,
    stealers: Vec<Stealer<Message>>,
) {
    loop {
        // 按照优先级寻找任务
        match find_task(&local_queue, &global_injector, &stealers) {
            Some(Message::NewJob(job)) => {
                job(); // 执行任务
            }
            Some(Message::Terminate) => {
                break; // 收到关闭信号，退出循环
            }
            None => {
                // 如果在所有地方都找不到任务，就让出CPU时间片，避免空转
                thread::yield_now();
            }
        }
    }
}

// 核心的“寻找任务”算法
fn find_task<'a>(
    local_queue: &'a DequeWorker<Message>,
    global_injector: &'a Arc<Injector<Message>>,
    stealers: &'a [Stealer<Message>],
) -> Option<Message> {
    // 优先级1：处理自己的本地任务 (LIFO)
    if let Some(job) = local_queue.pop() {
        return Some(job);
    }

    // 优先级2：从全局注入器获取任务 (FIFO)
    // --- 修改开始 ---
    match global_injector.steal() {
        crossbeam_deque::Steal::Success(job) => return Some(job),
        _ => (), // 对 Empty 和 Retry 情况什么都不做
    }
    // --- 修改结束 ---

    // 优先级3：从其他随机一个 worker 窃取任务 (FIFO)
    let mut rng = rand::thread_rng();
    let mut indices: Vec<usize> = (0..stealers.len()).collect();
    indices.shuffle(&mut rng);

    for &i in &indices {
        // --- 修改开始 ---
        match stealers[i].steal() {
            crossbeam_deque::Steal::Success(job) => return Some(job),
            _ => (), // 对 Empty 和 Retry 情况什么都不做
        }
        // --- 修改结束 ---
    }

    None
}