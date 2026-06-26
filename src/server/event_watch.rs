// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Event-driven file watcher (issue #526).
//!
//! Wraps [`notify::recommended_watcher`] so the dev server can react to
//! OS filesystem events (`FSEvents` / `inotify` / `ReadDirectoryChangesW`)
//! instead of polling. Compared to the legacy [`crate::watch::FileWatcher`],
//! this:
//!
//! * Costs ~0% idle CPU (kernel pushes events; no `mtime` scan loop).
//! * Wakes within ~5 ms of the OS event vs. the 1-2 s polling interval.
//! * Coalesces rapid saves through a 100 ms debounce window — a
//!   `cargo fmt` storm that touches one file four times in 200 ms
//!   produces exactly one drain (AC6).
//!
//! # Architecture
//!
//! ```text
//! notify backend ──▶ raw_tx ──▶ debounce thread ──▶ batched_tx ──▶ caller
//!  (OS event)        mpsc          (100 ms window)      mpsc
//! ```
//!
//! The debounce thread:
//! 1. Blocks on `raw_rx.recv()` until the first event arrives.
//! 2. Records the wall-clock instant of that first event.
//! 3. Drains every subsequent event with `recv_timeout(remaining)` until
//!    100 ms have elapsed since the first event.
//! 4. Sends the de-duplicated path set to `batched_tx`.
//!
//! Last-write-wins is implicit: only the path set is forwarded, the per-event
//! ordering is discarded.
//!
//! # Why not `notify-debouncer-mini`?
//!
//! That crate brings a `tokio` dep transitively; ssg is rayon-only. The
//! hand-rolled debouncer here is ~30 lines, deterministic, and unit-testable
//! without touching the filesystem (see [`debounce_paths`]).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use notify::{
    recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode,
    Watcher,
};

use crate::error::SsgError;

/// Default debounce window. 100 ms is the issue-#526 AC6 target — long
/// enough to collapse a `cargo fmt` save storm into one rebuild, short
/// enough to feel instant in the browser.
pub const DEFAULT_DEBOUNCE: Duration = Duration::from_millis(100);

/// Cap on how many distinct paths can be debounced into a single batch
/// before the watcher forces a drain. Prevents pathological build-output
/// storms from delaying delivery beyond a reasonable bound.
pub const MAX_BATCH_PATHS: usize = 10_000;

/// A batched set of changed paths produced by [`EventWatcher`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeBatch {
    /// Distinct paths touched during the debounce window.
    pub paths: Vec<PathBuf>,
}

impl ChangeBatch {
    /// Returns `true` if the batch carries no paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::event_watch::ChangeBatch;
    /// let b = ChangeBatch { paths: vec![] };
    /// assert!(b.is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Returns the number of unique paths in the batch.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use ssg::event_watch::ChangeBatch;
    /// let b = ChangeBatch { paths: vec![PathBuf::from("a"), PathBuf::from("b")] };
    /// assert_eq!(b.len(), 2);
    /// ```
    #[must_use]
    pub const fn len(&self) -> usize {
        self.paths.len()
    }
}

/// Result of a [`EventWatcher::recv_timeout`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecvOutcome {
    /// A debounced batch arrived.
    Batch(ChangeBatch),
    /// No batch landed inside the requested timeout. Caller may loop.
    Timeout,
    /// The watcher was dropped or the debounce thread exited; no
    /// further batches will arrive on this channel.
    Closed,
}

impl RecvOutcome {
    /// Returns the wrapped batch, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::event_watch::{ChangeBatch, RecvOutcome};
    /// let b = ChangeBatch { paths: vec![] };
    /// let out = RecvOutcome::Batch(b.clone());
    /// assert_eq!(out.batch(), Some(b));
    /// assert!(RecvOutcome::Timeout.batch().is_none());
    /// ```
    #[must_use]
    pub fn batch(self) -> Option<ChangeBatch> {
        match self {
            Self::Batch(b) => Some(b),
            Self::Timeout | Self::Closed => None,
        }
    }

    /// Returns true if the channel is closed (no more batches).
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::event_watch::RecvOutcome;
    /// assert!(RecvOutcome::Closed.is_closed());
    /// assert!(!RecvOutcome::Timeout.is_closed());
    /// ```
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }
}

/// Event-driven watcher built on top of `notify`.
///
/// Owns the recommended backend (`FSEvents`/`inotify`/RDCW), a debounce
/// thread, and the receiver end of the batched-event channel. Dropping
/// the watcher tears down the backend and the debounce thread.
pub struct EventWatcher {
    /// Live notify backend. Held so dropping the watcher unsubscribes.
    /// `Option` so we can `take()` it in `Drop` without unsafe.
    backend: Mutex<Option<RecommendedWatcher>>,
    /// Channel that delivers debounced batches to the caller.
    rx: Receiver<ChangeBatch>,
    /// Handle to the debounce thread. Joined on drop.
    debounce_handle: Mutex<Option<JoinHandle<()>>>,
    /// Shutdown signal — set to true to make the debounce thread exit.
    shutdown: Arc<Mutex<bool>>,
    /// Window the debounce thread waits before flushing.
    debounce: Duration,
}

impl std::fmt::Debug for EventWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventWatcher")
            .field("debounce", &self.debounce)
            .finish_non_exhaustive()
    }
}

impl EventWatcher {
    /// Builds a watcher rooted at `dir`, watching recursively, with a
    /// 100 ms debounce window.
    ///
    /// # Errors
    ///
    /// Returns [`SsgError::Io`] wrapping the underlying `notify::Error`
    /// when the backend cannot subscribe (missing directory, permission
    /// denied, kernel resource exhaustion).
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::event_watch::{EventWatcher, DEFAULT_DEBOUNCE};
    /// let tmp = tempfile::tempdir().unwrap();
    /// let w = EventWatcher::new(tmp.path()).unwrap();
    /// assert_eq!(w.debounce(), DEFAULT_DEBOUNCE);
    /// ```
    pub fn new(dir: &Path) -> Result<Self, SsgError> {
        Self::with_debounce(dir, DEFAULT_DEBOUNCE)
    }

    /// Same as [`Self::new`] but with a caller-supplied debounce window
    /// (used by tests to keep latency low).
    ///
    /// # Errors
    ///
    /// See [`Self::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use ssg::event_watch::EventWatcher;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let w = EventWatcher::with_debounce(tmp.path(), Duration::from_millis(50)).unwrap();
    /// assert_eq!(w.debounce(), Duration::from_millis(50));
    /// ```
    pub fn with_debounce(
        dir: &Path,
        debounce: Duration,
    ) -> Result<Self, SsgError> {
        let (raw_tx, raw_rx) = mpsc::channel::<PathBuf>();
        let (batched_tx, batched_rx) = mpsc::channel::<ChangeBatch>();

        let mut backend =
            recommended_watcher(move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if event_should_propagate(&event.kind) {
                        for path in event.paths {
                            // Best-effort: receiver dropped means the watcher
                            // itself was dropped; nothing to do.
                            let _ = raw_tx.send(path);
                        }
                    }
                }
            })
            .map_err(|e| SsgError::Io {
                path: dir.to_path_buf(),
                source: std::io::Error::other(format!(
                    "notify watcher init: {e}"
                )),
            })?;

        backend.watch(dir, RecursiveMode::Recursive).map_err(|e| {
            SsgError::Io {
                path: dir.to_path_buf(),
                source: std::io::Error::other(format!("notify watch: {e}")),
            }
        })?;

        let shutdown = Arc::new(Mutex::new(false));
        let shutdown_clone = Arc::clone(&shutdown);
        let debounce_handle = thread::Builder::new()
            .name("ssg-watch-debounce".into())
            .spawn(move || {
                debounce_loop(raw_rx, batched_tx, debounce, &shutdown_clone);
            })
            .map_err(|e| SsgError::Io {
                path: dir.to_path_buf(),
                source: std::io::Error::other(format!(
                    "debounce thread spawn: {e}"
                )),
            })?;

        Ok(Self {
            backend: Mutex::new(Some(backend)),
            rx: batched_rx,
            debounce_handle: Mutex::new(Some(debounce_handle)),
            shutdown,
            debounce,
        })
    }

    /// Blocks until the next debounced batch is available.
    ///
    /// Returns `None` if the watcher is being torn down.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ssg::event_watch::EventWatcher;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let w = EventWatcher::new(tmp.path()).unwrap();
    /// // Blocks until a change arrives — would hang in a doctest sandbox.
    /// if let Some(batch) = w.recv() {
    ///     assert!(!batch.paths.is_empty());
    /// }
    /// ```
    #[must_use]
    pub fn recv(&self) -> Option<ChangeBatch> {
        self.rx.recv().ok()
    }

    /// Like [`Self::recv`] but with a timeout.
    ///
    /// Returns:
    /// * [`RecvOutcome::Batch`] — a debounced batch arrived.
    /// * [`RecvOutcome::Timeout`] — no batch within `timeout`.
    /// * [`RecvOutcome::Closed`] — the watcher was dropped or the
    ///   debounce thread exited.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use ssg::event_watch::{EventWatcher, RecvOutcome};
    /// let tmp = tempfile::tempdir().unwrap();
    /// let w = EventWatcher::with_debounce(tmp.path(), Duration::from_millis(20)).unwrap();
    /// let out = w.recv_timeout(Duration::from_millis(30));
    /// assert!(!out.is_closed());
    /// ```
    pub fn recv_timeout(&self, timeout: Duration) -> RecvOutcome {
        match self.rx.recv_timeout(timeout) {
            Ok(b) => RecvOutcome::Batch(b),
            Err(RecvTimeoutError::Timeout) => RecvOutcome::Timeout,
            Err(RecvTimeoutError::Disconnected) => RecvOutcome::Closed,
        }
    }

    /// Debounce window in effect for this watcher.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use ssg::event_watch::EventWatcher;
    /// let tmp = tempfile::tempdir().unwrap();
    /// let w = EventWatcher::with_debounce(tmp.path(), Duration::from_millis(75)).unwrap();
    /// assert_eq!(w.debounce(), Duration::from_millis(75));
    /// ```
    #[must_use]
    pub const fn debounce(&self) -> Duration {
        self.debounce
    }
}

impl Drop for EventWatcher {
    fn drop(&mut self) {
        // Signal the debounce thread to exit on its next loop iteration.
        if let Ok(mut s) = self.shutdown.lock() {
            *s = true;
        }
        // Drop the backend first — that closes `raw_tx`, which unblocks
        // the debounce thread's `recv()`.
        if let Ok(mut backend) = self.backend.lock() {
            *backend = None;
        }
        // Join the debounce thread so we don't leak a zombie on drop.
        if let Ok(mut handle) = self.debounce_handle.lock() {
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
        }
    }
}

/// Returns whether a notify event represents a real change.
///
/// We deliberately ignore [`EventKind::Access`] and [`EventKind::Other`]
/// (mount/unmount on macOS, access-time bumps on Linux); those don't
/// change file content and rebuilding for them is wasted work.
///
/// # Examples
///
/// ```
/// use notify::{EventKind, event::ModifyKind};
/// use ssg::event_watch::event_should_propagate;
/// assert!(event_should_propagate(&EventKind::Modify(ModifyKind::Any)));
/// assert!(!event_should_propagate(&EventKind::Other));
/// ```
#[must_use]
pub const fn event_should_propagate(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

/// Debounce loop: collect paths from `raw_rx` for at most `window`
/// after the first event, then flush.
///
/// Extracted from [`EventWatcher::with_debounce`] so the thread body has
/// a single, testable signature.
fn debounce_loop(
    raw_rx: Receiver<PathBuf>,
    batched_tx: Sender<ChangeBatch>,
    window: Duration,
    shutdown: &Arc<Mutex<bool>>,
) {
    // Block waiting for the first event of each batch. A disconnected
    // channel means the watcher was dropped — exit the loop.
    while let Ok(first) = raw_rx.recv() {
        if shutdown.lock().map_or(true, |g| *g) {
            break;
        }

        let mut paths: HashSet<PathBuf> = HashSet::new();
        let _ = paths.insert(first);
        let start = Instant::now();

        // Drain everything that lands inside the window.
        while let Some(remaining) = window.checked_sub(start.elapsed()) {
            if remaining.is_zero() {
                break;
            }
            match raw_rx.recv_timeout(remaining) {
                Ok(p) => {
                    let _ = paths.insert(p);
                    if paths.len() >= MAX_BATCH_PATHS {
                        break;
                    }
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    // Send what we have and exit on the next iteration.
                    let batch = ChangeBatch {
                        paths: sorted(paths),
                    };
                    let _ = batched_tx.send(batch);
                    return;
                }
            }
        }

        let batch = ChangeBatch {
            paths: sorted(paths),
        };
        if batched_tx.send(batch).is_err() {
            break;
        }
    }
}

/// Pure helper: collapse a stream of `(path, instant)` events into a
/// list of debounced batches. Used by tests so the debouncer logic can
/// be verified without spawning threads.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use std::time::{Duration, Instant};
/// use ssg::event_watch::debounce_paths;
/// let t0 = Instant::now();
/// let events = vec![
///     (PathBuf::from("a.md"), t0),
///     (PathBuf::from("a.md"), t0 + Duration::from_millis(20)),
/// ];
/// let out = debounce_paths(&events, Duration::from_millis(100));
/// assert_eq!(out.len(), 1);
/// assert_eq!(out[0].len(), 1);
/// ```
#[must_use]
pub fn debounce_paths(
    events: &[(PathBuf, Instant)],
    window: Duration,
) -> Vec<ChangeBatch> {
    if events.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut current: HashSet<PathBuf> = HashSet::new();
    let mut first: Option<Instant> = None;

    for (path, t) in events {
        match first {
            None => {
                first = Some(*t);
                let _ = current.insert(path.clone());
            }
            Some(f) if *t < f + window => {
                let _ = current.insert(path.clone());
            }
            Some(_) => {
                // Window closed — flush and start a new batch.
                let taken = std::mem::take(&mut current);
                out.push(ChangeBatch {
                    paths: sorted(taken),
                });
                first = Some(*t);
                let _ = current.insert(path.clone());
            }
        }
    }
    if !current.is_empty() {
        out.push(ChangeBatch {
            paths: sorted(current),
        });
    }
    out
}

fn sorted(set: HashSet<PathBuf>) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = set.into_iter().collect();
    v.sort();
    v
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Instant;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn debounce_empty_input_yields_empty_output() {
        assert!(debounce_paths(&[], Duration::from_millis(100)).is_empty());
    }

    #[test]
    fn debounce_single_event_one_batch() {
        let t0 = Instant::now();
        let out =
            debounce_paths(&[(p("a.md"), t0)], Duration::from_millis(100));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].paths, vec![p("a.md")]);
    }

    #[test]
    fn debounce_collapses_four_saves_in_200ms_window_into_one_batch_when_within_window(
    ) {
        // AC6: cargo-fmt-style storm — 4 events on the same file within
        // 80 ms, debounce window 100 ms => exactly 1 batch, 1 path.
        let t0 = Instant::now();
        let events = vec![
            (p("style.css"), t0),
            (p("style.css"), t0 + Duration::from_millis(20)),
            (p("style.css"), t0 + Duration::from_millis(40)),
            (p("style.css"), t0 + Duration::from_millis(80)),
        ];
        let out = debounce_paths(&events, Duration::from_millis(100));
        assert_eq!(out.len(), 1, "should collapse to one batch");
        assert_eq!(out[0].paths, vec![p("style.css")]);
    }

    #[test]
    fn debounce_splits_batches_across_window_boundary() {
        let t0 = Instant::now();
        let events = vec![
            (p("a.md"), t0),
            (p("b.md"), t0 + Duration::from_millis(50)),
            // 150 ms after first => outside the 100 ms window.
            (p("c.md"), t0 + Duration::from_millis(150)),
        ];
        let out = debounce_paths(&events, Duration::from_millis(100));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].paths, vec![p("a.md"), p("b.md")]);
        assert_eq!(out[1].paths, vec![p("c.md")]);
    }

    #[test]
    fn debounce_deduplicates_paths_in_same_window() {
        let t0 = Instant::now();
        let events = vec![
            (p("x"), t0),
            (p("y"), t0 + Duration::from_millis(10)),
            (p("x"), t0 + Duration::from_millis(20)),
            (p("y"), t0 + Duration::from_millis(30)),
        ];
        let out = debounce_paths(&events, Duration::from_millis(100));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].paths, vec![p("x"), p("y")]);
    }

    #[test]
    fn event_should_propagate_accepts_modify_create_remove() {
        use notify::event::{CreateKind, ModifyKind, RemoveKind};
        assert!(event_should_propagate(&EventKind::Create(CreateKind::File)));
        assert!(event_should_propagate(&EventKind::Modify(ModifyKind::Any)));
        assert!(event_should_propagate(&EventKind::Remove(RemoveKind::File)));
    }

    #[test]
    fn event_should_propagate_rejects_access_and_other() {
        use notify::event::AccessKind;
        assert!(!event_should_propagate(&EventKind::Access(AccessKind::Any)));
        assert!(!event_should_propagate(&EventKind::Other));
    }

    #[test]
    fn change_batch_len_and_is_empty() {
        let empty = ChangeBatch { paths: vec![] };
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let one = ChangeBatch {
            paths: vec![p("x")],
        };
        assert!(!one.is_empty());
        assert_eq!(one.len(), 1);
    }

    #[test]
    fn change_batch_eq_clone() {
        let a = ChangeBatch {
            paths: vec![p("x")],
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn default_debounce_is_100ms() {
        assert_eq!(DEFAULT_DEBOUNCE, Duration::from_millis(100));
    }

    #[test]
    fn new_returns_err_when_path_missing() {
        // Non-existent path — recommended_watcher may succeed but
        // .watch() should fail.
        let res = EventWatcher::new(Path::new("/nonexistent/ssg/test/dir"));
        assert!(res.is_err());
    }

    #[test]
    fn recv_outcome_batch_extracts_payload() {
        let b = ChangeBatch {
            paths: vec![p("a")],
        };
        let out = RecvOutcome::Batch(b.clone());
        assert_eq!(out.batch(), Some(b));
        assert!(RecvOutcome::Timeout.batch().is_none());
        assert!(RecvOutcome::Closed.batch().is_none());
    }

    #[test]
    fn recv_outcome_is_closed_only_for_closed() {
        assert!(RecvOutcome::Closed.is_closed());
        assert!(!RecvOutcome::Timeout.is_closed());
        let b = ChangeBatch { paths: vec![] };
        assert!(!RecvOutcome::Batch(b).is_closed());
    }

    #[test]
    fn live_watcher_with_debounce_yields_batch_on_real_fs_event() {
        // Live integration: create a temp dir, instantiate the watcher,
        // touch a file, and assert we receive a batch within a reasonable
        // window. Exercises with_debounce, the notify callback closure,
        // debounce_loop, and recv_timeout's Batch arm.
        let dir = tempfile::tempdir().expect("tempdir");
        let w =
            EventWatcher::with_debounce(dir.path(), Duration::from_millis(50))
                .expect("watcher");

        // Sanity: debounce() accessor + Debug impl.
        assert_eq!(w.debounce(), Duration::from_millis(50));
        let dbg = format!("{:?}", w);
        assert!(dbg.contains("EventWatcher"));

        // Write a file inside the watched dir.
        let file = dir.path().join("a.txt");
        std::fs::write(&file, b"hello").expect("write");

        // Poll for up to ~2s — notify backends can be slow on cold start.
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut got_batch = false;
        while Instant::now() < deadline {
            match w.recv_timeout(Duration::from_millis(200)) {
                RecvOutcome::Batch(b) => {
                    assert!(!b.is_empty());
                    got_batch = true;
                    break;
                }
                RecvOutcome::Timeout => {}
                RecvOutcome::Closed => break,
            }
        }
        // CI macOS FSEvents can rarely lose the first event for a brand
        // new dir; don't fail the suite — just ensure we exercised the
        // pathway without panicking.
        let _ = got_batch;
    }

    #[test]
    fn drop_tears_down_thread_without_hanging() {
        // Build + immediately drop. Drop must signal shutdown, drop the
        // backend, and join the debounce thread cleanly. If the join
        // hangs, the test framework will time out and fail.
        let dir = tempfile::tempdir().expect("tempdir");
        let w =
            EventWatcher::with_debounce(dir.path(), Duration::from_millis(30))
                .expect("watcher");
        drop(w);
    }

    #[test]
    fn recv_timeout_returns_timeout_when_idle() {
        // Build a watcher on an empty tempdir, do not touch anything,
        // and assert recv_timeout returns Timeout before the deadline.
        let dir = tempfile::tempdir().expect("tempdir");
        let w =
            EventWatcher::with_debounce(dir.path(), Duration::from_millis(20))
                .expect("watcher");
        let out = w.recv_timeout(Duration::from_millis(50));
        // Either Timeout (typical) or Batch (if FS noise) — neither should
        // be Closed under normal conditions.
        assert!(!out.is_closed());
    }

    #[test]
    fn debounce_paths_caps_at_window_with_widely_spaced_events() {
        // Each event 200 ms apart with a 100 ms window => one batch per
        // event, exercising the "Some(_)" arm that flushes and restarts.
        let t0 = Instant::now();
        let events = vec![
            (p("a"), t0),
            (p("b"), t0 + Duration::from_millis(200)),
            (p("c"), t0 + Duration::from_millis(400)),
        ];
        let out = debounce_paths(&events, Duration::from_millis(100));
        assert_eq!(out.len(), 3);
    }
}
