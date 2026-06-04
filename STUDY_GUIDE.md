# Worker Pool V1 — Deep Study Guide

**Target:** ~3–4 sessions, 25–45 min each
**Goal:** Understand every line, every tradeoff, every failure mode
**Method:** Pomodoros. Do one section per session. Stop when the timer ends — even mid-sentence.

---

## Session 1: Ownership, Closures, and the Job Type

### Mini Lesson (10 min)

Open `src/job.rs` and `src/threadpool.rs`.

```rust
pub type Job = Box<dyn FnOnce() + Send + 'static>;
```

This is a **type alias** for a boxed closure. Deconstruct it piece by piece:

| Piece | Meaning |
|---|---|
| `Box<...>` | Heap-allocated. The size is unknown at compile time. |
| `dyn FnOnce()` | Trait object — any closure that can be called once. `FnOnce` is the calling convention. |
| `Send` | Can be transferred to another thread. Required because `thread::spawn` needs `Send`. |
| `'static` | No borrowed data. The closure owns everything it captures. Required because `thread::spawn` needs `'static`. |

Without **any one** of these bounds, the code would not compile.

**Try it:** Comment out `Send` from `job.rs` and run `cargo build`. Read the error. Then put it back and comment out `'static`. Read that error too.

### 🔍 Debug Exercise (10 min)

```rust
// Why does this NOT compile? Fix it.
let pool = ThreadPool::new(2);
let local_data = String::from("hello");
pool.execute(|| {
    println!("{}", local_data);
});
```

Hint: What bound is on the closure? Does `local_data` satisfy it?

### Quiz (5 min — answer aloud or write it down)

1. What would happen if `Job` used `Fn` instead of `FnOnce`? Would anything break?
2. Why does `Box<dyn FnOnce() + Send + 'static>` need the `Box` at all? Why can't the channel send `dyn FnOnce() + Send + 'static` directly?
3. In `execute`, the closure `f` is moved into `Box::new(f)`. After that, can `f` be used again? Why or why not?

### 🛠 Fix a Broken Feature (10 min)

Here is a version of `execute` that looks correct but panics at runtime. Identify the bug and fix it:

```rust
pub fn execute<F>(&self, f: F)
where
    F: FnOnce() + Send + 'static,
{
    self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    self.metrics.increment_submitted();
}
```

What scenario causes the panic? When does it happen?

### Read These

- Rust Book, [Ch 10.2: Traits as Parameters](https://doc.rust-lang.org/stable/book/ch10-02-traits.html) — skip to "Trait Bound Syntax"
- Rust Book, [Ch 16.1: Threads](https://doc.rust-lang.org/stable/book/ch16-01-threads.html) — just the `thread::spawn` signature
- Rust Book, [Ch 19.4: Advanced Functions & Closures](https://doc.rust-lang.org/stable/book/ch19-05-advanced-functions-and-closures.html) — section "Returning Closures"

---

## Session 2: Channels, Shared State, and the Lock-While-Blocking Problem

### Mini Lesson (10 min)

Open `src/worker.rs` and `src/threadpool.rs`.

The channel setup:

```
ThreadPool creates (sender, receiver)      // mpsc = multi-producer, single-consumer
                ↓
         sender goes to ThreadPool          // &self — anyone can call execute
                ↓
         receiver goes into Arc<Mutex<>>    // shared by all workers
                ↓
         each worker calls recv()           // blocks until a job arrives
```

**Key constraint:** `mpsc::Receiver` is **not** `Clone` and **not** `Sync`. You cannot give each worker its own receiver. The `Arc<Mutex<>>` is the workaround.

**The problem:** When a worker does `receiver.lock().unwrap().recv()`, the `MutexGuard` lives as long as the temporary expression — which includes the entire duration of `recv()`. `recv()` **blocks** until a job arrives. So only **one** worker at a time can wait for jobs. The rest are blocked on the mutex.

Think of it like a single cashier window with a rope maze — only one person can reach the window at a time, even if others are just standing there.

### 🔍 Debug Exercise (15 min)

Print evidence of the serialization. Add this to the worker spawn:

```rust
let start = std::time::Instant::now();
while let Ok(job) = receiver.lock().unwrap().recv() {
    let waited = start.elapsed();
    println!("Worker {} waited {:?} before getting a job", _id, waited);
    // ... rest of the loop
}
```

Submit 16 jobs to a pool of 4 workers. Look at the wait times. Do workers wait for each other? Or can multiple wait simultaneously?

(You need the `_id` back — change the struct to keep it, or use some other marker.)

### Quiz (5 min)

1. What happens if you remove the `Arc` and give each `Worker::new` a fresh `Mutex::new(receiver)`? (Think: does the channel work with multiple receivers?)
2. `mpsc` stands for "multi-producer, single-consumer." What constraint does the "single-consumer" part impose on our design?
3. Why doesn't `while let Ok(job) = receiver.lock().unwrap().recv()` hold the lock while the job runs? Trace the temporary lifetimes.

### 🛠 Fix a Broken Feature (10 min)

Here's a version that deadlocks. Why?

```rust
// threadpool.rs
pub fn shutdown(&mut self) {
    for worker in &mut self.workers {
        worker.join();   // wait for worker to finish
    }
    drop(self.sender.take());  // THEN close the channel
}
```

When does the deadlock happen? Does it happen every time? What's the minimum number of jobs needed to trigger it?

### Read These

- Rust Book, [Ch 16.2: Message Passing](https://doc.rust-lang.org/stable/book/ch16-02-message-passing.html)
- Rust Book, [Ch 16.3: Shared State](https://doc.rust-lang.org/stable/book/ch16-03-shared-state.html)
- [Rustnomicon: Temporary Scopes](https://doc.rust-lang.org/nomicon/destructors.html) — just the part about temporary lifetime extension

---

## Session 3: Atomic Ordering and the Metrics Gap

### Mini Lesson (10 min)

Open `src/metrics.rs`.

The struct uses three atomics with `Ordering::Relaxed`:

```rust
self.jobs_submitted.fetch_add(1, Ordering::Relaxed);
self.jobs_completed.fetch_add(1, Ordering::Relaxed);
self.total_runtime_micros.fetch_add(x, Ordering::Relaxed);
```

`Relaxed` means: "This value will eventually be consistent, but there is **no ordering guarantee** relative to other operations." In practice:

- Thread A calls `increment_submitted()`.
- Thread B calls `jobs_submitted()`.
- Thread B might see the increment immediately — or it might not, for a few nanoseconds.

This is fine for a shutdown summary (we print after all threads have joined). But `pending_jobs()` reads two separate atomics:

```rust
pub fn pending_jobs(&self) -> usize {
    let submitted = self.jobs_submitted.load(Ordering::Relaxed);
    let completed = self.jobs_completed.load(Ordering::Relaxed);
    submitted.saturating_sub(completed)
}
```

Because the two loads are separate, the observed state is **inconsistent**:
- Thread sees `submitted = 10` (written after completed), `completed = 5` (written before submitted) → `pending = 5` ✓
- Thread sees `submitted = 5` (stale), `completed = 10` (fresh) → `pending = 0` (should be "negative", clamped to 0) ✗

The fix would be `Ordering::Acquire` / `Ordering::Release` or a single atomic pair, but that's for a later session. For now, understand the gap.

### 🔍 Debug Exercise (15 min)

Add an assertion in `shutdown()` that fires if `jobs_completed > jobs_submitted`:

```rust
// This should be impossible — but can it happen?
assert!(
    self.metrics.jobs_completed() <= self.metrics.jobs_submitted(),
    "completed > submitted: {} > {}",
    self.metrics.jobs_completed(),
    self.metrics.jobs_submitted(),
);
```

Run `cargo test` 50 times in a loop:

```bash
for i in $(seq 1 50); do cargo test test_high_volume_stress --quiet; done
```

Does the assertion ever fire? If not, increase the job count to 10000 and try again. Note: even if it doesn't fire here, the **possibility** exists on different hardware/OS. Relaxed ordering is not about "does it break on my machine" — it's about "does the C++ memory model allow it."

### Quiz (5 min)

1. What is the *minimum* guarantee provided by `Ordering::Relaxed`? What is the *maximum*?
2. If you changed every `load` to `Ordering::Acquire` and every `store` to `Ordering::Release`, what would change about `pending_jobs()`? Would it be correct?
3. In `add_runtime`, we use `fetch_add`. What would break if we used `store(load() + x, Ordering::Relaxed)` instead?

### 🛠 Fix a Broken Feature (10 min)

The `average_runtime_ms` method divides by zero if `completed == 0`. We handle that with `if completed == 0 { return 0.0; }`.

But what if `completed == 1` and the runtime is 500µs? What if `completed == 1` and runtime overflows `u64`? How many jobs would it take to overflow a `u64` counter at 1ms per job?

```rust
// u64 max ≈ 1.8 * 10^19
// 1ms per job → 10^3 jobs/s
// Years to overflow:
```

Compute this and write the answer in the file as a comment. Put it next to `total_runtime_micros`.

### Read These

- [Rust Atomics and Locks](https://marabos.nl/atomics/) — **Chapter 2: Atomics** (free online). Read the section on `Relaxed` ordering specifically. This is the single best resource on the topic.
- The embedded Rustonomicon has a shorter [section on atomics](https://doc.rust-lang.org/nomicon/atomics.html) if the book is too long.
- Rust Book, [Ch 16.4: `Sync` and `Send`](https://doc.rust-lang.org/stable/book/ch16-04-extensible-concurrency-sync-and-send.html) — 3 pages, skim-able.

---

## Session 4: Drop, Panic Safety, and the Flaky Test

### Mini Lesson (10 min)

Open `src/threadpool.rs` and `src/lib.rs`.

**Two shutdown paths:**

1. **Explicit:** `pool.shutdown()` — drops sender, joins workers, **prints diagnostics**. Called in `main.rs`.
2. **Implicit:** `drop(pool)` / end of scope — calls `Drop::drop`, which drains and joins but **does not print**. Called automatically.

The `Drop` impl must not print (surprising side effect). It must not panic (double-panic is abort). And it must not deadlock.

**Panic propagation:**

In `worker.rs:join()`:

```rust
if let Err(e) = thread.join() {
    std::panic::resume_unwind(e);
}
```

If a worker panics, `thread.join()` returns `Err(Box<dyn Any>)`. We call `resume_unwind(e)`, which re-throws the **original** panic payload with its **original message**. This is why `test_job_panic_behavior` can match `#[should_panic(expected = "Intentional System Disruption")]`.

Without `resume_unwind`, using `.unwrap()` instead, the panic message becomes `"called 'Result::unwrap()' on an 'Err' value: Any { ... }"` — losing the original message.

**The flaky test:**

```rust
pool.execute(|| panic!("Intentional System Disruption"));
std::thread::sleep(Duration::from_millis(20));
pool.shutdown();
```

Timeline A (test passes):
1. Worker calls `recv()`, gets the panic job.
2. Worker panics. Thread join returns `Err(...)`. `resume_unwind` propagates. `shutdown()` panics. ✓

Timeline B (test fails):
1. Worker hasn't called `recv()` yet.
2. `shutdown()` drops the sender. Channel closes.
3. Worker calls `recv()` → `Err(RecvError)`. Worker exits cleanly.
4. `thread.join()` returns `Ok(())`. `shutdown()` returns normally. ✗ no panic.

The `sleep(20ms)` makes Timeline A more likely but does not guarantee it.

### 🔍 Debug Exercise (15 min)

Prove the flakiness exists. Run this test 200 times:

```bash
for i in $(seq 1 200); do
    cargo test test_job_panic_behavior --quiet 2>/dev/null
    if [ $? -ne 0 ]; then echo "FAIL at run $i"; fi
done
```

(It might not fail on your machine — the timing window depends on scheduler, cores, load. If it doesn't fail in 200, run 2000. If still no failure, that's interesting — why might this be hard to trigger on your specific system?)

Then add a `println!` to `shutdown()` to trace the actual sequence:

```rust
println!("shutdown: sender.is_some={}", self.sender.is_some());
```

Run the panic test once with `--nocapture`:

```bash
cargo test test_job_panic_behavior -- --nocapture 2>&1
```

### Quiz (5 min)

1. Why does `Drop::drop` call `shutdown()` in the current code? What happens if we remove the `Drop` impl entirely?
2. In `worker.rs:join()`, we use `resume_unwind`. What would happen if the test used `#[should_panic(expected = "some other message")]` instead? Would the test fail or pass?
3. What is a "double panic" and when does Rust abort instead of unwind?

### 🛠 Fix a Broken Feature (15 min)

The flaky test can be fixed without adding complex synchronization. Here's one approach: make `Worker::join` **not** panic on worker panic, but instead store the panic. Then `shutdown` checks for stored panics.

```rust
// Fill in the blanks
pub struct Worker {
    thread: Option<thread::JoinHandle<()>>,
    // What field goes here? What type?
}

pub fn join(&mut self) {
    // Should this still resume_unwind?
    // Or should it store the panic for later?
}

pub fn panicked(&self) -> bool {
    // Return whether this worker caught a panic
}
```

Then in the test, instead of relying on `shutdown` to panic:

```rust
pool.shutdown();
// Check that at least one worker caught a panic
```

This eliminates the race: it doesn't matter whether the worker got the job before or after the sender dropped — if it ran the job and panicked, we detect it. If it never got the job, no panic happened, and we can check for that too.

### Read These

- [Rust Book, Ch 9.3: `panic!` or Not?](https://doc.rust-lang.org/stable/book/ch09-03-to-panic-or-not-to-panic.html)
- [Rust Reference: Panic in Drop](https://doc.rust-lang.org/reference/destructors.html#panic-in-destructors) — one paragraph, but important
- [std::panic::resume_unwind docs](https://doc.rust-lang.org/std/panic/fn.resume_unwind.html) — read the "Panics" section

---

## Session 5: The Lock-While-Blocking Fix

This session is a **deep-dive implementation** into the main bottleneck in the code.

### Mini Lesson (10 min)

**Observation:** The `Arc<Mutex<mpsc::Receiver<Job>>>` pattern serializes all workers at the `recv()` call. Only one worker can block on the channel at a time.

**Approach:** Replace the single shared channel with per-worker channels.

Instead of:

```
(sender, receiver) — one pair shared via Arc<Mutex<>>
```

Do:

```
for each worker:
    (tx, rx) — one pair per worker, no mutex needed

ThreadPool holds N senders (one per worker)
Worker holds its own receiver (no Arc, no Mutex)
```

**Round-robin dispatch:** When `execute` is called, pick a worker's sender. Simplest: cycle through them.

```rust
pub fn execute<F>(&self, f: F)
where
    F: FnOnce() + Send + 'static,
{
    // pick next sender in round-robin
    // send job
}
```

**Tradeoffs:**
- ✅ No mutex contention on dequeue
- ✅ Workers can block independently
- ❌ If one worker's queue fills up (if bounded), jobs to that worker block even if other workers are idle — worse than shared queue under load
- ❌ Load distribution is naive round-robin (not work-stealing)

**For a study exercise,** this is the next iteration (V2 on your roadmap). Implement it and compare.

### 🔍 Implementation Exercise (30 min)

Implement the per-worker channel approach:

1. Change `ThreadPool` to hold a `Vec<mpsc::Sender<Job>>` instead of a single `Option<mpsc::Sender<Job>>`.
2. Remove the `Arc<Mutex<>>` from the receiver. Each `Worker` gets an `mpsc::Receiver<Job>` directly — no lock needed.
3. Add a round-robin counter to `ThreadPool`.
4. In `execute`, send to `senders[next % len]` and advance the counter.
5. In `shutdown`, drop all senders, then join all workers.

**Starter signature for Worker:**

```rust
impl Worker {
    pub fn new(
        receiver: mpsc::Receiver<Job>,
        metrics: Arc<PerformanceMetrics>,
    ) -> Self { ... }
}
```

No more `Arc<Mutex<>>`, no more `_id`.

**Check your work:**
- All existing tests should still pass.
- The `test_job_panic_behavior` test should still be flaky (we haven't fixed that yet — but think about whether the race window changed).

### Quiz (5 min)

1. In the per-worker channel design, what happens to the `shutdown()` sequence? Do you still need to drop the senders before joining workers?
2. Without the mutex, is the per-worker-channel design faster under high concurrency? What about under low concurrency?
3. In a round-robin dispatch, if one worker gets a long job, the next 3 jobs go to other workers. After 4 short jobs, the long one is still running. Is this better or worse than the shared queue? Why?

### Read These

- [std::sync::mpsc](https://doc.rust-lang.org/std/sync/mpsc/index.html) — skim the docs, note the "single-consumer" part
- [crossbeam-channel](https://docs.rs/crossbeam-channel) — docs only, don't install it yet. Compare its API to `mpsc`. What does `crossbeam` offer that `mpsc` doesn't? (Spoiler: multi-consumer, select, etc.)
- *Optional:* [Locks Aren't Slow; Lock Contention Is](https://preshing.com/20111118/locks-arent-slow-lock-contention-is/) — C++ focused but the principle applies universally.

---

## Final Integration Challenge

Run all the following tasks. If any fail, investigate.

```bash
# Clean build — zero warnings
cargo clean && cargo build 2>&1

# All unit + integration tests
cargo test 2>&1

# Run with race-heavy concurrency
cargo test --test thread_pool_concurrent -- --test-threads=16 2>&1

# Build in release mode (more optimization, different timing)
cargo test --release 2>&1

# Run the binary
cargo run 2>&1
```

Then summarize in a file `INSIGHTS.md`:

```
## What I learned
-
-
-

## What surprised me
-
-
-

## What I still don't understand
-
-
-
```

Bring the questions you don't understand to the next session.
