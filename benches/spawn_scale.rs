//! Large-scale task spawning benchmarks.
//! Tests how the runtime scales with varying numbers of spawned tasks,
//! measuring both spawn latency and memory overhead patterns.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::{mpsc, Barrier};

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
}

fn single_rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

// Large-scale empty task spawning
fn bench_spawn_1k_empty(c: &mut Criterion) {
    const NUM_TASKS: usize = 1_000;
    let rt = rt();

    c.bench_function("spawn_1k_empty", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for _ in 0..NUM_TASKS {
                    handles.push(tokio::spawn(async {}));
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            });
        })
    });
}

fn bench_spawn_10k_empty(c: &mut Criterion) {
    const NUM_TASKS: usize = 10_000;
    let rt = rt();

    c.bench_function("spawn_10k_empty", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for _ in 0..NUM_TASKS {
                    handles.push(tokio::spawn(async {}));
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            });
        })
    });
}

fn bench_spawn_100k_empty(c: &mut Criterion) {
    const NUM_TASKS: usize = 100_000;
    let rt = rt();

    c.bench_function("spawn_100k_empty", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for _ in 0..NUM_TASKS {
                    handles.push(tokio::spawn(async {}));
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            });
        })
    });
}

// Batched spawning with synchronization
fn bench_spawn_batched_sync(c: &mut Criterion) {
    const NUM_TASKS: usize = 10_000;
    const BATCH_SIZE: usize = 100;
    let rt = rt();

    c.bench_function("spawn_batched_sync", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (tx, mut rx) = mpsc::channel(NUM_TASKS);
                let completed = Arc::new(AtomicUsize::new(0));

                for batch in 0..(NUM_TASKS / BATCH_SIZE) {
                    let mut batch_handles = Vec::with_capacity(BATCH_SIZE);

                    for i in 0..BATCH_SIZE {
                        let tx = tx.clone();
                        let completed = completed.clone();
                        let task_id = batch * BATCH_SIZE + i;

                        batch_handles.push(tokio::spawn(async move {
                            // Simulate some work
                            tokio::task::yield_now().await;

                            if completed.fetch_add(1, Ordering::Relaxed) + 1 == NUM_TASKS {
                                tx.send(()).await.unwrap();
                            }
                            task_id
                        }));
                    }

                    // Don't wait for this batch, let them run concurrently
                    tokio::spawn(async move {
                        for handle in batch_handles {
                            handle.await.unwrap();
                        }
                    });
                }

                // Wait for all tasks to complete
                rx.recv().await.unwrap();
                black_box(completed.load(Ordering::Relaxed));
            });
        })
    });
}

// Memory-intensive tasks to test allocation patterns
fn bench_spawn_memory_intensive(c: &mut Criterion) {
    const NUM_TASKS: usize = 1_000;
    let rt = rt();

    c.bench_function("spawn_memory_intensive", |b| {
        b.iter(|| {
            rt.block_on(async {
                let barrier = Arc::new(Barrier::new(NUM_TASKS + 1));
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for i in 0..NUM_TASKS {
                    let barrier = barrier.clone();

                    handles.push(tokio::spawn(async move {
                        // Allocate some memory to simulate real workload
                        let data = vec![i; 1000]; // 4KB per task

                        barrier.wait().await;

                        // Do some work with the data
                        data.iter().sum::<usize>()
                    }));
                }

                // Wait for all tasks to be ready
                barrier.wait().await;

                // Collect results
                let mut total = 0;
                for handle in handles {
                    total += handle.await.unwrap();
                }

                black_box(total);
            });
        })
    });
}

// Single-threaded runtime scaling
fn bench_single_thread_scale_1k(c: &mut Criterion) {
    const NUM_TASKS: usize = 1_000;
    let rt = single_rt();

    c.bench_function("single_thread_scale_1k", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for i in 0..NUM_TASKS {
                    handles.push(tokio::spawn(async move {
                        tokio::task::yield_now().await;
                        i
                    }));
                }

                let mut sum = 0;
                for handle in handles {
                    sum += handle.await.unwrap();
                }

                black_box(sum);
            });
        })
    });
}

fn bench_single_thread_scale_10k(c: &mut Criterion) {
    const NUM_TASKS: usize = 10_000;
    let rt = single_rt();

    c.bench_function("single_thread_scale_10k", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for i in 0..NUM_TASKS {
                    handles.push(tokio::spawn(async move {
                        tokio::task::yield_now().await;
                        i
                    }));
                }

                let mut sum = 0;
                for handle in handles {
                    sum += handle.await.unwrap();
                }

                black_box(sum);
            });
        })
    });
}

// Task spawn rate limiting
fn bench_spawn_rate_limited(c: &mut Criterion) {
    const NUM_TASKS: usize = 5_000;
    const SPAWN_DELAY: Duration = Duration::from_micros(10);
    let rt = rt();

    c.bench_function("spawn_rate_limited", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (tx, mut rx) = mpsc::channel(1);
                let completed = Arc::new(AtomicUsize::new(0));
                let completed_clone = completed.clone();

                tokio::spawn(async move {
                    for i in 0..NUM_TASKS {
                        let completed = completed_clone.clone();
                        let tx = tx.clone();

                        tokio::spawn(async move {
                            let val = i * 2;
                            if completed.fetch_add(1, Ordering::Relaxed) + 1 == NUM_TASKS {
                                tx.send(val).await.unwrap();
                            }
                        });

                        // Rate limiting
                        tokio::time::sleep(SPAWN_DELAY).await;
                    }
                });

                let final_val = rx.recv().await.unwrap();
                black_box(final_val);
                black_box(completed.load(Ordering::Relaxed));
            });
        })
    });
}

// Mixed workload: some tasks yield, some compute, some wait
fn bench_spawn_mixed_workload(c: &mut Criterion) {
    const NUM_TASKS: usize = 2_000;
    let rt = rt();

    c.bench_function("spawn_mixed_workload", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for i in 0..NUM_TASKS {
                    handles.push(tokio::spawn(async move {
                        match i % 3 {
                            0 => {
                                // Yield task
                                for _ in 0..5 {
                                    tokio::task::yield_now().await;
                                }
                                i
                            }
                            1 => {
                                // Compute task
                                let mut sum = 0;
                                for j in 0..100 {
                                    sum += i + j;
                                }
                                sum
                            }
                            _ => {
                                // Wait task
                                tokio::time::sleep(Duration::from_micros(1)).await;
                                i * 2
                            }
                        }
                    }));
                }

                let mut total = 0;
                for handle in handles {
                    total += handle.await.unwrap();
                }

                black_box(total);
            });
        })
    });
}

criterion_group!(
    spawn_scaling,
    bench_spawn_1k_empty,
    bench_spawn_10k_empty,
    bench_spawn_100k_empty,
    bench_spawn_batched_sync,
    bench_spawn_memory_intensive,
    bench_single_thread_scale_1k,
    bench_single_thread_scale_10k,
    bench_spawn_rate_limited,
    bench_spawn_mixed_workload,
);

criterion_main!(spawn_scaling);
