// benches/std_threadpool_benchmark.rs

use criterion::{black_box, Criterion};
use std::sync::mpsc;
use threadpool::ThreadPool as StdThreadPool; // 使用 as 区分

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub fn benchmark_std_threadpool(c: &mut Criterion) {
    c.bench_function("StdThreadPool_4_threads", |b| {
        let pool = StdThreadPool::new(4);
        let (tx, rx) = mpsc::channel();

        b.iter(|| {
            for _ in 0..10 {
                let tx_clone = tx.clone();
                pool.execute(move || {
                    let result = fibonacci(black_box(20));
                    tx_clone.send(result).unwrap();
                });
            }
            for _ in 0..10 {
                rx.recv().unwrap();
            }
        });
    });
}