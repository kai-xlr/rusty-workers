# Worker Pool V1

A multi-threaded thread pool in Rust with runtime observability.

Part of the [Rust Systems Engineering Roadmap](https://github.com/anomalyco/opencode) — Phase 1 Concurrency Core System.

## Architecture

```
src/
├── job.rs          Box<dyn FnOnce() + Send + 'static> type alias
├── worker.rs       Thread-per-worker, loops on recv(), tracks metrics
├── threadpool.rs   Manages workers, channel, shutdown lifecycle
├── metrics.rs      PerformanceMetrics — atomics for observability
└── main.rs         Demo: 20 jobs across 4 workers
```

- **`Job`** — `Box<dyn FnOnce() + Send + 'static>`
- **`Worker`** — Owns a thread that pulls jobs from a shared `Arc<Mutex<mpsc::Receiver>>`, records runtime per job
- **`ThreadPool`** — Creates workers, dispatches jobs via `mpsc::Sender`, provides graceful shutdown with diagnostics
- **`PerformanceMetrics`** — Atomic counters for jobs submitted, completed, pending, and average runtime

## API

```rust
let mut pool = ThreadPool::new(4);
pool.execute(|| println!("hello"));
pool.shutdown();
// Drop also drains and joins — silently
```

## Build & Run

```bash
cargo run
```

On shutdown, prints a diagnostic summary:

```
Workers Configured    : 4
Total Jobs Enqueued   : 20
Total Jobs Completed  : 20
Pending Tasks Abandoned: 0
Average Job Runtime   : 15.12 ms
```

## Tests

```bash
cargo test
```

| Test | What it checks |
|------|---------------|
| `test_enqueue_success` | Execute does not panic |
| `test_single_job_execution` | One job, flag flips |
| `test_high_volume_stress` | 100 jobs, correct count + metrics |
| `test_clean_shutdown` | Shutdown completes < 200ms |
| `test_job_panic_behavior` | Worker panic propagates through join |

Integration tests cover shared receiver pipelines, sequential execution, and concurrent dispatch.

## Study

See [`STUDY_GUIDE.md`](./STUDY_GUIDE.md) for a session-by-session deep-dive with mini lessons, quizzes, and debugging exercises.
