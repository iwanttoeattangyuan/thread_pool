// benches/my_pool_benchmark.rs

use criterion::{black_box, Criterion};
use hyper_thread_pool::ThreadPool; // 导入库
use std::sync::mpsc;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// 注意函数是 pub 的，这样 main.rs 才能调用它
pub fn benchmark_my_pool(c: &mut Criterion) {
    c.bench_function("MyThreadPool_4_threads", |b| {
        let pool = ThreadPool::new(4);
        let (tx, rx) = mpsc::channel();

        b.iter(|| {
            // 每次迭代运行10个任务
            for _ in 0..10 {
                let tx_clone = tx.clone();
                pool.execute(move || {
                    let result = fibonacci(black_box(20)); // 使用一个稍大的计算量
                    tx_clone.send(result).unwrap();
                });
            }
            // 等待所有任务完成
            for _ in 0..10 {
                rx.recv().unwrap();
            }
        });
    });
}