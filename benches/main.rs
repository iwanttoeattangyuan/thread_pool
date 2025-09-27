// benches/main.rs

use criterion::criterion_group;
use criterion::criterion_main;

// 1. 将其他文件声明为模块
mod my_threadpool_benchmark;
mod rayon_benchmark;
mod io_threadpool_benchmark;

// 2. 从模块中导入 benchmark 函数
use my_threadpool_benchmark::benchmark_my_pool;
use rayon_benchmark::{benchmark_rayon_par_iter, benchmark_rayon_scope};
use io_threadpool_benchmark::benchmark_std_threadpool;

// 3. 将所有 benchmark 函数组合成一个 group
criterion_group!(
    benches,
    benchmark_my_pool,
    benchmark_std_threadpool,
    benchmark_rayon_scope,
    benchmark_rayon_par_iter
);

// 4. 指定 main 函数入口
criterion_main!(benches);