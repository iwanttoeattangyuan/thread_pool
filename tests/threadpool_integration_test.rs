use thread_pool::ThreadPool;
use std::panic;
use std::time::Duration;

fn add(a: i32, b: i32) -> i32 {
    a + b
}


#[test]
fn test_thread_pool() {
    let pool = ThreadPool::new(4);
    for i in 0..8 {
        pool.execute(move || {
            let result = add(i, 2);
            println!("Task {i} result: {result}");
        });
    }
    println!("main thread is doing other work");

}

#[test]
fn test_job_panics() {
    // 这个测试验证了即使一个任务 panic，线程池本身也不会崩溃
    let pool = ThreadPool::new(4);

    let result = panic::catch_unwind(|| {
        pool.execute(|| {
            panic!("this job is designed to panic!");
        });
        
        // 给一点时间让 panic 的任务被执行
        thread::sleep(Duration::from_secs(1));
    });

    // 验证主线程没有因为 worker 的 panic 而 panic
    assert!(result.is_ok()); 
    // 在一个更复杂的测试中，你可以继续派发任务，验证线程池是否依然可用
}
// 在 tests 模块中
#[test]
fn test_graceful_shutdown() {
    let (tx, rx) = mpsc::channel();
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        let tx_clone = tx.clone();
        pool.execute(move || {
            // 模拟工作
            thread::sleep(Duration::from_millis(100));
            // 发送完成信号
            tx_clone.send(i).unwrap();
        });
    }

    // 关键：在这里显式 drop(pool)，触发 drop 逻辑
    drop(pool);

    // 验证：在 pool 被销毁后，我们应该能收到所有 8 个任务的完成信号
    let mut results = Vec::new();
    for _ in 0..8 {
        results.push(rx.recv().unwrap());
    }
    
    results.sort(); // 排序以确保结果一致
    assert_eq!(results, (0..8).collect::<Vec<_>>());
}

#[test]
#[should_panic]
fn test_new_with_zero_threads() {
    ThreadPool::new(0);
}
