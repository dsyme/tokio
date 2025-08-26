//! Benchmark task lifecycle operations including joining and cancellation.
//! These benchmarks measure the performance of JoinHandle operations beyond
//! just spawning tasks, covering the full task lifecycle.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::time::sleep;

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

// Task join performance - measures JoinHandle::await cost
fn bench_task_join_immediate(c: &mut Criterion) {
    let rt = rt();

    c.bench_function("task_join_immediate", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handle = tokio::spawn(async { 42 });
                black_box(handle.await.unwrap());
            });
        })
    });
}

fn bench_task_join_after_yield(c: &mut Criterion) {
    let rt = rt();

    c.bench_function("task_join_after_yield", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handle = tokio::spawn(async {
                    tokio::task::yield_now().await;
                    42
                });
                black_box(handle.await.unwrap());
            });
        })
    });
}

fn bench_task_join_multiple(c: &mut Criterion) {
    let rt = rt();
    const NUM_TASKS: usize = 100;

    c.bench_function("task_join_multiple", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for i in 0..NUM_TASKS {
                    handles.push(tokio::spawn(async move { i }));
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

// Task cancellation performance
fn bench_task_abort_immediate(c: &mut Criterion) {
    let rt = rt();

    c.bench_function("task_abort_immediate", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handle = tokio::spawn(async {
                    // Task that would run for a long time
                    loop {
                        sleep(Duration::from_secs(1)).await;
                    }
                });

                handle.abort();
                let result = handle.await;
                black_box(result.is_err()); // Should be cancelled
            });
        })
    });
}

fn bench_task_abort_after_start(c: &mut Criterion) {
    let rt = rt();

    c.bench_function("task_abort_after_start", |b| {
        b.iter(|| {
            rt.block_on(async {
                let counter = Arc::new(AtomicUsize::new(0));
                let counter_clone = counter.clone();

                let handle = tokio::spawn(async move {
                    loop {
                        counter_clone.fetch_add(1, Ordering::Relaxed);
                        tokio::task::yield_now().await;
                    }
                });

                // Let task run briefly
                tokio::task::yield_now().await;

                handle.abort();
                let result = handle.await;
                black_box(result.is_err()); // Should be cancelled
                black_box(counter.load(Ordering::Relaxed)); // Track work done
            });
        })
    });
}

fn bench_task_abort_batch(c: &mut Criterion) {
    let rt = rt();
    const NUM_TASKS: usize = 100;

    c.bench_function("task_abort_batch", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_TASKS);

                for _ in 0..NUM_TASKS {
                    handles.push(tokio::spawn(async {
                        loop {
                            sleep(Duration::from_millis(1)).await;
                        }
                    }));
                }

                // Abort all tasks
                for handle in &handles {
                    handle.abort();
                }

                // Wait for all aborts to complete
                for handle in handles {
                    let result = handle.await;
                    black_box(result.is_err());
                }
            });
        })
    });
}

// Task completion vs cancellation race scenarios
fn bench_task_completion_race(c: &mut Criterion) {
    let rt = rt();

    c.bench_function("task_completion_race", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Task that completes quickly vs slow abort
                let handle = tokio::spawn(async {
                    tokio::task::yield_now().await;
                    42
                });

                // Race between natural completion and abort
                let result = tokio::select! {
                    value = handle => value.unwrap(),
                    _ = sleep(Duration::from_micros(1)) => {
                        // This branch would abort if task takes too long
                        -1
                    }
                };

                black_box(result);
            });
        })
    });
}

// Single-threaded runtime join performance
fn bench_single_thread_join(c: &mut Criterion) {
    let rt = single_rt();

    c.bench_function("single_thread_task_join", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handle = tokio::spawn(async {
                    tokio::task::yield_now().await;
                    42
                });
                black_box(handle.await.unwrap());
            });
        })
    });
}

fn bench_single_thread_abort(c: &mut Criterion) {
    let rt = single_rt();

    c.bench_function("single_thread_task_abort", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handle = tokio::spawn(async {
                    loop {
                        sleep(Duration::from_secs(1)).await;
                    }
                });

                handle.abort();
                let result = handle.await;
                black_box(result.is_err());
            });
        })
    });
}

criterion_group!(
    task_lifecycle,
    bench_task_join_immediate,
    bench_task_join_after_yield,
    bench_task_join_multiple,
    bench_task_abort_immediate,
    bench_task_abort_after_start,
    bench_task_abort_batch,
    bench_task_completion_race,
    bench_single_thread_join,
    bench_single_thread_abort,
);

criterion_main!(task_lifecycle);
