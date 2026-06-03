# Worker Pool V1

A multi-threaded thread pool implementation in Rust.

Supports spawning a fixed set of worker threads that pull jobs from a shared
message-passing channel, execute them concurrently, and shut down gracefully.

## Architecture

- **`Job`** — Type alias for `Box<dyn FnOnce() + Send + 'static>`
- **`Worker`** — Owns a single thread that loops on `receiver.recv()`, executing
  each job until the channel is closed
- **`ThreadPool`** — Manages a collection of `Worker`s and a `mpsc::Sender<Job>`
  - `new(size)` — Creates `size` workers sharing a receiver via `Arc<Mutex<…>>`
  - `execute(f)` — Boxes and sends a closure to the channel
  - `shutdown()` — Drops the sender so workers break out of `recv()`, then joins
    all threads
  - `Drop` — Automatically calls `shutdown()`

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | `Job`, `Worker`, `ThreadPool` definitions |
| `src/main.rs` | Demo: dispatches 20 jobs to a 4-worker pool |
| `tests/job_allocation.rs` | Job allocation & execution |
| `tests/single_worker_channel.rs` | Single worker, sequential jobs |
| `tests/worker_clean_shutdown.rs` | Worker execute + clean shutdown |
| `tests/multi_worker_pipeline.rs` | 3 workers sharing one receiver |
| `tests/thread_pool_concurrent.rs` | ThreadPool executes 20 concurrent jobs |

## Build & Run

```bash
cargo run
```

## Tests

```bash
cargo test
```
