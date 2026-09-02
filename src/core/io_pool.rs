// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Bounded writer-thread pool that decouples disk writes from rayon
//! CPU workers (issue #569, phase 1).
//!
//! Rayon worker threads that call `fs::write` directly stall a CPU
//! slot for the duration of the syscall. [`IoPool`] moves those
//! writes onto a small dedicated pool of writer threads (2–4) fed by
//! a **bounded** `std::sync::mpsc` channel: producers enqueue
//! `{path, bytes}` jobs with [`IoPool::write`] and the bounded
//! channel provides natural backpressure (a full queue blocks the
//! sender instead of buffering unbounded memory).
//!
//! # Design constraints
//!
//! - **std-only, tokio-free** — per
//!   [ADR-0001](../../docs/adr/0001-tokio-free.md), `ssg` runs one
//!   scheduler (rayon) plus plain OS threads; no async executor is
//!   introduced here.
//! - **`io_uring` is out of scope** — that is phase 2 of issue #569
//!   (v0.0.48+, Linux-only feature flag). This module is the
//!   thread-pool backend only.
//! - **No silent data loss** — every write error is captured and
//!   surfaced by [`IoPool::flush`]. Dropping the pool without a
//!   final `flush()` still drains and joins the writers; any errors
//!   that were never observed via `flush()` are logged at `error`
//!   level from `Drop`.
//!
//! # Flush semantics
//!
//! [`IoPool::flush`] is a *barrier*, not a shutdown: it blocks until
//! every job enqueued so far has been fully processed (written or
//! failed), then reports the first captured error (logging any
//! additional ones). The pool remains usable afterwards, so a build
//! phase can `flush()` between batches and reuse the same threads.
//!
//! # Examples
//!
//! ```rust
//! use ssg::io_pool::IoPool;
//! use tempfile::tempdir;
//!
//! let dir = tempdir().unwrap();
//! let pool = IoPool::new();
//! pool.write(dir.path().join("a.html"), b"<p>a</p>".to_vec()).unwrap();
//! pool.write(dir.path().join("b.html"), b"<p>b</p>".to_vec()).unwrap();
//! pool.flush().unwrap(); // barrier: both files are durably on disk
//! assert_eq!(std::fs::read(dir.path().join("a.html")).unwrap(), b"<p>a</p>");
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

use crate::error::{PathErrorExt, SsgError};

/// Queue capacity per writer thread. Small enough that a slow disk
/// exerts backpressure on producers quickly, large enough to keep
/// the writers busy between producer bursts.
const QUEUE_CAP_PER_WORKER: usize = 32;

/// Hard ceiling on writer threads — disk write throughput saturates
/// with very few writers; more threads only add seek contention.
const MAX_WRITERS: usize = 4;

/// A single queued write: destination path plus the full contents.
#[derive(Debug)]
struct WriteJob {
    path: PathBuf,
    bytes: Vec<u8>,
}

/// Mutable pool state shared between producers, workers, and
/// `flush()` waiters.
#[derive(Debug, Default)]
struct StateInner {
    /// Jobs enqueued but not yet fully processed (written or failed).
    pending: usize,
    /// Write failures captured since the last `flush()`.
    errors: Vec<(PathBuf, io::Error)>,
    /// Successfully completed writes since pool creation.
    completed: usize,
}

/// Shared synchronization block: state guarded by a mutex plus the
/// condvar `flush()` waits on.
#[derive(Debug)]
struct PoolState {
    inner: Mutex<StateInner>,
    all_done: Condvar,
}

impl PoolState {
    /// Locks the inner state, recovering from a poisoned mutex.
    ///
    /// Workers never panic while holding the lock (the write happens
    /// outside the critical section), so poison recovery is safe: the
    /// counters are always internally consistent.
    fn lock(&self) -> MutexGuard<'_, StateInner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A small pool of dedicated writer threads fed by a bounded channel.
///
/// See the [module docs](self) for the full design rationale
/// (issue #569 phase 1, ADR-0001).
///
/// # Examples
///
/// ```rust
/// use ssg::io_pool::IoPool;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// let pool = IoPool::with_threads(2);
/// pool.write(dir.path().join("page.html"), b"<html/>".to_vec()).unwrap();
/// pool.flush().unwrap();
/// assert!(dir.path().join("page.html").exists());
/// ```
#[derive(Debug)]
pub struct IoPool {
    /// `Some` while the pool is live; taken (dropped) in `Drop` to
    /// close the channel and let workers drain + exit.
    tx: Option<SyncSender<WriteJob>>,
    /// Writer thread handles, joined in `Drop`.
    workers: Vec<JoinHandle<()>>,
    /// Shared pending/error accounting.
    state: Arc<PoolState>,
}

impl Default for IoPool {
    fn default() -> Self {
        Self::new()
    }
}

impl IoPool {
    /// Creates a pool with the default writer count:
    /// `min(4, max(1, available_parallelism / 2))`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::io_pool::IoPool;
    ///
    /// let pool = IoPool::new();
    /// pool.flush().unwrap(); // empty pool flushes trivially
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_threads(default_writer_threads())
    }

    /// Creates a pool with an explicit writer-thread count.
    ///
    /// `threads` is clamped to the `1..=4` range: zero writers would
    /// deadlock producers, and more than four writers only adds seek
    /// contention on the output disk.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::io_pool::IoPool;
    ///
    /// let pool = IoPool::with_threads(0); // clamped to 1
    /// pool.flush().unwrap();
    /// ```
    #[must_use]
    pub fn with_threads(threads: usize) -> Self {
        let threads = threads.clamp(1, MAX_WRITERS);
        let (tx, rx) = sync_channel::<WriteJob>(threads * QUEUE_CAP_PER_WORKER);
        let rx = Arc::new(Mutex::new(rx));
        let state = Arc::new(PoolState {
            inner: Mutex::new(StateInner::default()),
            all_done: Condvar::new(),
        });

        let workers = (0..threads)
            .map(|i| spawn_writer(i, Arc::clone(&rx), Arc::clone(&state)))
            .filter_map(|handle| match handle {
                Ok(h) => Some(h),
                Err(e) => {
                    log::error!("io_pool: failed to spawn writer thread: {e}");
                    None
                }
            })
            .collect::<Vec<_>>();

        // If *no* thread could be spawned, fall back to a degenerate
        // pool whose `write` performs the I/O inline — never deadlock.
        Self {
            tx: if workers.is_empty() { None } else { Some(tx) },
            workers,
            state,
        }
    }

    /// Enqueues a write of `bytes` to `path`.
    ///
    /// Blocks when the bounded queue is full (backpressure). The
    /// write itself happens asynchronously on a writer thread; any
    /// failure is captured and reported by the next [`flush`].
    ///
    /// Returns an error only if the job could not be enqueued at all
    /// (all writer threads gone); in the degenerate zero-worker
    /// fallback the write is performed inline instead.
    ///
    /// [`flush`]: IoPool::flush
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::io_pool::IoPool;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let pool = IoPool::new();
    /// pool.write(dir.path().join("x.txt"), b"x".to_vec()).unwrap();
    /// pool.flush().unwrap();
    /// ```
    pub fn write(
        &self,
        path: impl Into<PathBuf>,
        bytes: Vec<u8>,
    ) -> Result<(), SsgError> {
        let path = path.into();
        let Some(tx) = self.tx.as_ref() else {
            // Degenerate fallback: no writer threads — write inline
            // so no job is ever silently dropped.
            perform_write(&path, &bytes).with_path(&path)?;
            self.state.lock().completed += 1;
            return Ok(());
        };

        // Count the job as pending *before* it enters the queue so a
        // concurrent `flush()` cannot slip past it.
        self.state.lock().pending += 1;

        if let Err(e) = tx.send(WriteJob { path, bytes }) {
            // Channel disconnected: workers are gone. Undo the
            // accounting and surface a typed error.
            let mut inner = self.state.lock();
            inner.pending -= 1;
            if inner.pending == 0 {
                self.state.all_done.notify_all();
            }
            drop(inner);
            let job = e.0;
            return Err(SsgError::Io {
                path: job.path,
                source: io::Error::other(
                    "io_pool: writer threads terminated; job not enqueued",
                ),
            });
        }
        Ok(())
    }

    /// Barrier: blocks until every job enqueued so far is fully
    /// processed, then reports write failures.
    ///
    /// Returns the **first** captured error (with its path); any
    /// additional failures are logged at `error` level so nothing is
    /// silently dropped. The error buffer is cleared, and the pool
    /// stays alive — `flush()` does **not** shut the pool down and
    /// may be called repeatedly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::io_pool::IoPool;
    ///
    /// let pool = IoPool::new();
    /// // Writing into a directory that does not exist fails at flush.
    /// pool.write("/nonexistent-ssg-dir/x.txt", b"x".to_vec()).unwrap();
    /// assert!(pool.flush().is_err());
    /// // The pool remains usable after a failed flush.
    /// pool.flush().unwrap();
    /// ```
    pub fn flush(&self) -> Result<(), SsgError> {
        let mut inner = self.state.lock();
        while inner.pending > 0 {
            inner = self
                .state
                .all_done
                .wait(inner)
                .unwrap_or_else(PoisonError::into_inner);
        }
        let errors = std::mem::take(&mut inner.errors);
        drop(inner);

        let mut iter = errors.into_iter();
        let Some((first_path, first_err)) = iter.next() else {
            return Ok(());
        };
        for (path, err) in iter {
            log::error!(
                "io_pool: additional write failure at '{}': {err}",
                path.display()
            );
        }
        Err(SsgError::Io {
            path: first_path,
            source: first_err,
        })
    }

    /// Number of writes that have completed successfully since the
    /// pool was created. Primarily useful for tests and diagnostics.
    ///
    /// Note: this is a live counter; call [`IoPool::flush`] first for
    /// a stable reading.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::io_pool::IoPool;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let pool = IoPool::new();
    /// assert_eq!(pool.completed_writes(), 0);
    /// pool.write(dir.path().join("y.txt"), b"y".to_vec()).unwrap();
    /// pool.flush().unwrap();
    /// assert_eq!(pool.completed_writes(), 1);
    /// ```
    #[must_use]
    pub fn completed_writes(&self) -> usize {
        self.state.lock().completed
    }

    /// Number of writer threads backing this pool.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ssg::io_pool::IoPool;
    ///
    /// assert_eq!(IoPool::with_threads(9).threads(), 4); // clamped
    /// assert!(IoPool::new().threads() >= 1);
    /// ```
    #[must_use]
    pub const fn threads(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for IoPool {
    /// Drains and joins: closes the channel so workers finish every
    /// queued job, then joins them. Errors never observed through
    /// [`IoPool::flush`] are logged — not silently discarded.
    fn drop(&mut self) {
        // Closing the sender makes `recv()` return `Err` once the
        // queue is empty, so workers drain naturally then exit.
        drop(self.tx.take());
        for handle in self.workers.drain(..) {
            if handle.join().is_err() {
                log::error!("io_pool: writer thread panicked");
            }
        }
        let inner = self.state.lock();
        for (path, err) in &inner.errors {
            log::error!(
                "io_pool: write failure at '{}' dropped without flush(): {err}",
                path.display()
            );
        }
    }
}

/// Spawns one named writer thread, with a fault-injection point so
/// tests can drive the "no writer threads could be spawned"
/// degenerate-pool fallback (feature `test-fault-injection`).
fn spawn_writer(
    index: usize,
    rx: Arc<Mutex<Receiver<WriteJob>>>,
    state: Arc<PoolState>,
) -> io::Result<JoinHandle<()>> {
    fail_point!("io_pool::spawn", |_| {
        Err(io::Error::other("injected: io_pool::spawn"))
    });
    std::thread::Builder::new()
        .name(format!("ssg-io-writer-{index}"))
        .spawn(move || worker_loop(&rx, &state))
}

/// Default writer-thread count: half the available cores, floored at
/// 1 and capped at [`MAX_WRITERS`].
fn default_writer_threads() -> usize {
    let cores = std::thread::available_parallelism()
        .map_or(2, std::num::NonZeroUsize::get);
    (cores / 2).clamp(1, MAX_WRITERS)
}

/// Writer-thread body: pull jobs until the channel closes.
fn worker_loop(rx: &Mutex<Receiver<WriteJob>>, state: &PoolState) {
    loop {
        // Hold the receiver lock only while dequeuing, never during
        // the write itself.
        let job = {
            let guard = rx.lock().unwrap_or_else(PoisonError::into_inner);
            guard.recv()
        };
        let Ok(job) = job else {
            return; // channel closed and drained — pool is dropping
        };

        let result = perform_write(&job.path, &job.bytes);

        let mut inner = state.lock();
        match result {
            Ok(()) => inner.completed += 1,
            Err(e) => inner.errors.push((job.path, e)),
        }
        inner.pending -= 1;
        if inner.pending == 0 {
            state.all_done.notify_all();
        }
    }
}

/// The actual disk write, with a fault-injection point mirroring the
/// naming convention used by the other fs paths
/// (`tests/fault_injection.rs`, feature `test-fault-injection`).
fn perform_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    fail_point!("io_pool::write", |_| {
        Err(io::Error::other("injected: io_pool::write"))
    });
    fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use tempfile::tempdir;

    /// Extracts the path from an `SsgError::Io`, `None` otherwise.
    ///
    /// Used instead of inline `match … => panic!` so both arms are
    /// exercised (see `io_error_path_returns_none_for_non_io`).
    fn io_error_path(err: &SsgError) -> Option<PathBuf> {
        match err {
            SsgError::Io { path, .. } => Some(path.clone()),
            _ => None,
        }
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn io_error_path_returns_none_for_non_io() {
        let err = SsgError::Validation {
            field: "f".to_string(),
            message: "m".to_string(),
        };
        assert!(io_error_path(&err).is_none());
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn default_pool_behaves_like_new() {
        let pool = IoPool::default();
        assert!(pool.threads() >= 1);
        assert!(pool.flush().is_ok());
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn flush_logs_additional_errors_and_returns_first() {
        crate::test_support::init_logger();
        let dir = tempdir().unwrap();
        let bad_a = dir.path().join("no-dir-a").join("a.html");
        let bad_b = dir.path().join("no-dir-b").join("b.html");

        let pool = IoPool::with_threads(1);
        pool.write(&bad_a, b"a".to_vec()).unwrap();
        pool.write(&bad_b, b"b".to_vec()).unwrap();

        // Both writes fail; flush returns the first error and logs
        // the second (the additional-failure loop body executes).
        let err = pool.flush().expect_err("flush must fail");
        let path = io_error_path(&err).expect("must be an Io error");
        assert!(path == bad_a || path == bad_b);

        // Error buffer is cleared afterwards.
        assert!(pool.flush().is_ok());
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn drop_without_flush_logs_unobserved_errors() {
        crate::test_support::init_logger();
        let dir = tempdir().unwrap();
        let bad = dir.path().join("missing-dir").join("x.html");
        {
            let pool = IoPool::with_threads(1);
            pool.write(&bad, b"x".to_vec()).unwrap();
            // Give the worker time to fail the write so the error is
            // buffered before the drop (drop drains regardless, but
            // this makes the captured-error path deterministic).
            while pool.state.lock().pending > 0 {
                std::thread::yield_now();
            }
            // No flush() — Drop must log the unobserved failure.
        }
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn empty_pool_flush_is_ok() {
        let pool = IoPool::new();
        assert!(pool.flush().is_ok());
        assert_eq!(pool.completed_writes(), 0);
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn thread_count_is_clamped() {
        assert_eq!(IoPool::with_threads(0).threads(), 1);
        assert_eq!(IoPool::with_threads(100).threads(), MAX_WRITERS);
        let d = default_writer_threads();
        assert!((1..=MAX_WRITERS).contains(&d));
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn concurrent_rayon_producers_all_bytes_correct() {
        let dir = tempdir().unwrap();
        let pool = IoPool::with_threads(3);
        let n = 200usize;

        (0..n)
            .into_par_iter()
            .try_for_each(|i| {
                pool.write(
                    dir.path().join(format!("f{i}.html")),
                    format!("<p>page {i}</p>").into_bytes(),
                )
            })
            .unwrap();

        pool.flush().unwrap();
        assert_eq!(pool.completed_writes(), n);

        for i in 0..n {
            let got = fs::read_to_string(dir.path().join(format!("f{i}.html")))
                .unwrap();
            assert_eq!(got, format!("<p>page {i}</p>"));
        }
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn write_error_surfaces_at_flush_and_pool_survives() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("no-such-subdir").join("x.html");

        let pool = IoPool::with_threads(2);
        pool.write(&missing, b"x".to_vec()).unwrap(); // enqueue OK
        let err = pool.flush().expect_err("flush must surface the failure");
        let path = io_error_path(&err)
            .expect("flush error must be SsgError::Io with a path");
        assert_eq!(path, missing);

        // Error buffer cleared; pool still functional.
        assert!(pool.flush().is_ok());
        pool.write(dir.path().join("ok.html"), b"ok".to_vec())
            .unwrap();
        pool.flush().unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("ok.html")).unwrap(),
            "ok"
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn unwritable_dir_error_surfaces_at_flush() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let ro = dir.path().join("readonly");
        fs::create_dir(&ro).unwrap();
        fs::set_permissions(&ro, fs::Permissions::from_mode(0o555)).unwrap();

        let pool = IoPool::with_threads(2);
        pool.write(ro.join("blocked.html"), b"x".to_vec()).unwrap();
        assert!(pool.flush().is_err());

        // Restore permissions so tempdir cleanup succeeds.
        fs::set_permissions(&ro, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn drop_without_flush_completes_queued_writes() {
        let dir = tempdir().unwrap();
        {
            let pool = IoPool::with_threads(2);
            for i in 0..50 {
                pool.write(
                    dir.path().join(format!("d{i}.txt")),
                    format!("v{i}").into_bytes(),
                )
                .unwrap();
            }
            // No flush — Drop must drain the queue and join.
        }
        for i in 0..50 {
            assert_eq!(
                fs::read_to_string(dir.path().join(format!("d{i}.txt")))
                    .unwrap(),
                format!("v{i}")
            );
        }
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn flush_is_reusable_across_batches() {
        let dir = tempdir().unwrap();
        let pool = IoPool::with_threads(2);

        pool.write(dir.path().join("a.txt"), b"a".to_vec()).unwrap();
        pool.flush().unwrap();
        assert_eq!(pool.completed_writes(), 1);

        pool.write(dir.path().join("b.txt"), b"b".to_vec()).unwrap();
        pool.flush().unwrap();
        assert_eq!(pool.completed_writes(), 2);

        assert_eq!(fs::read_to_string(dir.path().join("a.txt")).unwrap(), "a");
        assert_eq!(fs::read_to_string(dir.path().join("b.txt")).unwrap(), "b");
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn poisoned_state_lock_recovers_via_into_inner() {
        // `PoolState::lock` recovers from a poisoned mutex instead of
        // panicking (see its doc comment). Workers never panic while
        // holding the lock in normal operation, so the only way to
        // reach this branch in a test is to poison it directly.
        let state = Arc::new(PoolState {
            inner: Mutex::new(StateInner::default()),
            all_done: Condvar::new(),
        });
        let poison_state = Arc::clone(&state);
        let handle = std::thread::spawn(move || {
            let _guard = poison_state.inner.lock().unwrap();
            panic!("deliberately poison the mutex for test coverage");
        });
        assert!(handle.join().is_err());

        // Recovers instead of panicking; state is still readable.
        let guard = state.lock();
        assert_eq!(guard.pending, 0);
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn flush_condvar_wait_recovers_from_poison() {
        // `flush()`'s `all_done.wait(...).unwrap_or_else(...)` also
        // recovers from a poisoned mutex. Reproduced directly against
        // a bare `PoolState` (mirroring `flush`'s wait loop) since
        // driving it through a live `IoPool` cannot deterministically
        // poison the lock while a waiter is parked in `wait()`.
        let state = Arc::new(PoolState {
            inner: Mutex::new(StateInner {
                pending: 1,
                ..StateInner::default()
            }),
            all_done: Condvar::new(),
        });

        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
        let waiter_state = Arc::clone(&state);
        let waiter = std::thread::spawn(move || {
            let mut inner = waiter_state.lock();
            let _ = ready_tx.send(());
            while inner.pending > 0 {
                inner = waiter_state
                    .all_done
                    .wait(inner)
                    .unwrap_or_else(PoisonError::into_inner);
            }
            inner.pending
        });

        // The waiter signals only after acquiring the lock while
        // `pending == 1`, so it is guaranteed to call `Condvar::wait`
        // next (which releases the mutex while parked); poll until
        // that release is observable before poisoning it.
        ready_rx.recv().unwrap();
        while state.inner.try_lock().is_err() {
            std::thread::yield_now();
        }

        let poison_state = Arc::clone(&state);
        let poisoner = std::thread::spawn(move || {
            let mut inner = poison_state.inner.lock().unwrap();
            inner.pending = 0;
            poison_state.all_done.notify_all();
            panic!("poison the mutex while notifying the waiter");
        });
        assert!(poisoner.join().is_err());

        let pending_after =
            waiter.join().expect("waiter must recover, not panic");
        assert_eq!(pending_after, 0);
    }

    #[test]
    #[serial_test::parallel(io_pool_failpoints)]
    fn worker_loop_receiver_lock_recovers_from_poison() {
        // `worker_loop`'s `rx.lock().unwrap_or_else(...)` uses the same
        // poison-recovery idiom for the receiver mutex.
        let (tx, rx) = sync_channel::<WriteJob>(1);
        let rx = Arc::new(Mutex::new(rx));
        let poison_rx = Arc::clone(&rx);
        let handle = std::thread::spawn(move || {
            let _guard = poison_rx.lock().unwrap();
            panic!("deliberately poison the receiver mutex");
        });
        assert!(handle.join().is_err());

        drop(tx);
        let guard = rx.lock().unwrap_or_else(PoisonError::into_inner);
        assert!(guard.recv().is_err());
    }

    /// Fault-injection tests. Failpoints are process-global, so every
    /// test here serialises via `serial_test` and restores the
    /// failpoint with an RAII guard (mirrors `tests/fault_injection.rs`).
    #[cfg(feature = "test-fault-injection")]
    mod fault_injection {
        use super::*;
        use serial_test::serial;

        /// RAII guard that disables a failpoint on drop.
        struct FailGuard<'a>(&'a str);

        impl Drop for FailGuard<'_> {
            fn drop(&mut self) {
                let _ = fail::cfg(self.0, "off");
            }
        }

        #[test]
        #[serial(io_pool_failpoints)]
        fn injected_write_failure_surfaces_at_flush() {
            let _guard = FailGuard("io_pool::write");
            fail::cfg("io_pool::write", "return").expect("activate failpoint");

            let dir = tempdir().unwrap();
            let pool = IoPool::with_threads(1);
            pool.write(dir.path().join("x.html"), b"x".to_vec())
                .unwrap();
            let err = pool.flush().expect_err("injected write must fail");
            assert!(
                format!("{err}").contains("injected: io_pool::write"),
                "got: {err}"
            );
        }

        #[test]
        #[serial(io_pool_failpoints)]
        fn spawn_failure_falls_back_to_inline_writes() {
            crate::test_support::init_logger();
            let dir = tempdir().unwrap();

            let pool = {
                let _guard = FailGuard("io_pool::spawn");
                fail::cfg("io_pool::spawn", "return")
                    .expect("activate failpoint");
                IoPool::with_threads(2)
            };

            // No writer threads: the pool degrades to inline writes.
            assert_eq!(pool.threads(), 0);
            pool.write(dir.path().join("inline.html"), b"i".to_vec())
                .unwrap();
            assert_eq!(pool.completed_writes(), 1);
            assert_eq!(
                fs::read_to_string(dir.path().join("inline.html")).unwrap(),
                "i"
            );

            // An inline write that fails surfaces immediately.
            let err = pool
                .write(dir.path().join("no-dir").join("y.html"), b"y".to_vec())
                .expect_err("inline write into missing dir must fail");
            assert!(io_error_path(&err).is_some());

            assert!(pool.flush().is_ok());
        }

        #[test]
        #[serial(io_pool_failpoints)]
        fn write_after_workers_died_returns_typed_error() {
            crate::test_support::init_logger();
            let dir = tempdir().unwrap();
            let pool = IoPool::with_threads(1);

            {
                let _guard = FailGuard("io_pool::write");
                fail::cfg("io_pool::write", "panic")
                    .expect("activate failpoint");
                // The single worker dequeues this job and panics,
                // dropping the channel receiver.
                pool.write(dir.path().join("doomed.html"), b"d".to_vec())
                    .unwrap();

                // Once the receiver is gone, send() fails and write()
                // surfaces the disconnected-channel error.
                let err = loop {
                    match pool
                        .write(dir.path().join("after.html"), b"a".to_vec())
                    {
                        Err(e) => break e,
                        Ok(()) => std::thread::sleep(
                            std::time::Duration::from_millis(1),
                        ),
                    }
                };
                assert!(
                    format!("{err}").contains("writer threads terminated"),
                    "got: {err}"
                );
            }
            // Drop joins the panicked worker (join().is_err() branch).
            // Never call flush() here: the panicked job left
            // `pending > 0` permanently.
            drop(pool);
        }
    }
}
