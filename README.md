# Worker Pool v1

A multi-threaded worker pool / thread pool implementation in Rust.

## Current State

- **`src/counter.rs`** — Shared counter demo using `Arc<Mutex<u64>>` across 10 threads
- **`src/threadpool.rs`** — Thread-safe `JobQueue` backed by `Arc<Mutex<VecDeque>>`; queue type only, no thread pool yet
- **`src/job.rs`** — Stub (job type definition)
- **`src/worker.rs`** — Stub (worker thread logic)
- **`src/main.rs`** — Runs counter demo then executes jobs sequentially on the main thread

## Build & Run

```bash
cargo run
```
