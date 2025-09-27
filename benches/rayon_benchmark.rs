// benches/rayon_benchmark.rs

use criterion::{black_box, Criterion};
use rayon::prelude::*;
use std::sync::mpsc;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// 1. 模拟任务提交模式
pub fn benchmark_rayon_scope(c: &mut Criterion) {
    c.bench_function("Rayon_scope_simulation", |b| {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
        let (tx, rx) = mpsc::channel();

        b.iter(|| {
            pool.scope(|s| {
                for _ in 0..10 {
                    let tx_clone = tx.clone();
                    s.spawn(move |_| {
                        let result = fibonacci(black_box(20));
                        tx_clone.send(result).unwrap();
                    });
                }
            });
            for _ in 0..10 {
                rx.recv().unwrap();
            }
        });
    });
}

// 2. Rayon 惯用法：数据并行模式
pub fn benchmark_rayon_par_iter(c: &mut Criterion) {
    c.bench_function("Rayon_par_iter_idiomatic", |b| {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
        
        b.iter(|| {
            // 使用 pool.install 来确保这段代码在我们的线程池中运行
            pool.install(|| {
                let results: Vec<_> = (0..10)
                    .into_par_iter()
                    .map(|_| fibonacci(black_box(20)))
                    .collect();
                
                // 确保 results 被使用，防止被优化掉
                black_box(results);
            });
        });
    });
}