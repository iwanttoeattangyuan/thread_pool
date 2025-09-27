// benches/work_stealing_benchmark.rs

use criterion::{black_box, Criterion};
use thread_pool::ThreadPool;
use std::sync::mpsc;
use rayon::prelude::*; // 修正 #1: 导入 Rayon prelude

// Rayon 的等价实现
fn rayon_recursive_fib(n: u32) -> u64 {
    if n <= 20 {
        return fibonacci(n as u64);
    }
    let (left, right) = rayon::join(|| rayon_recursive_fib(n - 1), || rayon_recursive_fib(n - 2));
    left + right
}

// 基础的 fibonacci 计算
fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// 修正 #2: 删除了不再使用的 spawn_fib 函数

pub fn benchmark_work_stealing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Work Stealing Fibonacci");

    group.bench_function("MyThreadPool_work_stealing", |b| {
        let pool = ThreadPool::new(4);
        
        b.iter(|| {
            let (tx, rx) = mpsc::channel();
            let num_tasks = 100; // 提交大量任务
            for _ in 0..num_tasks {
                let tx_clone = tx.clone();
                pool.execute(move || {
                    tx_clone.send(fibonacci(black_box(20))).unwrap();
                });
            }
            
            // 等待所有任务完成
            for _ in 0..num_tasks {
                rx.recv().unwrap();
            }
        })
    });

    group.bench_function("Rayon_work_stealing", |b| {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
        b.iter(|| {
            pool.install(|| {
                let results: Vec<_> = (0..100) // 同样是100个任务
                    .into_par_iter() // 现在这行可以正确工作了
                    .map(|_| fibonacci(black_box(20)))
                    .collect();
                black_box(results);
            });
        })
    });

    group.finish();
}