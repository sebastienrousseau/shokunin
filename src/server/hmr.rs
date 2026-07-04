// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Hot-module-reload protocol for `ssg dev` (issue #526).
//!
//! Defines the WebSocket message format the dev server pushes to every
//! connected browser tab and the [`HmrBroadcaster`] that fans messages
//! out across N tabs.
//!
//! # Protocol (AC5)
//!
//! Each frame is a JSON object:
//!
//! ```json
//! { "type": "hmr-css",  "paths": ["assets/style.css"],  "sha": "abc..." }
//! { "type": "hmr-html", "paths": ["/blog/foo/"],         "sha": "def..." }
//! { "type": "reload",   "paths": [],                     "sha": "" }
//! ```
//!
//! * `type` — one of `hmr-css`, `hmr-html`, `reload`.
//!   * `hmr-css`: client swaps `<link>` href in-place (no reload, scroll
//!     preserved, no FOUC). AC2.
//!   * `hmr-html`: client fetches each path and swaps the `<main>` body
//!     (or `<body>` if no `<main>`) — scroll position preserved. AC3/AC4.
//!   * `reload`: client does a full `location.reload()` — used for
//!     `<head>` / config / build-error recovery.
//! * `paths` — affected paths. For `hmr-html` these are the rebuilt
//!   page URLs the dep graph reported invalidated.
//! * `sha` — short content hash so the client can de-dupe redundant
//!   frames from rapid saves. Optional; empty string is fine.
//!
//! Frames are pushed as Text WebSocket messages from
//! [`HmrBroadcaster::broadcast`]; the JS client in
//! `crate::livereload` parses them.
//!
//! # Architecture
//!
//! ```text
//! caller ──▶ broadcast() ──▶ for each registered Sender:
//!                              try_send(json) ──▶ tab thread ──▶ ws.send()
//! ```
//!
//! Each connected tab owns a `Sender<String>`; the broadcaster holds the
//! receiving side in a `Mutex<Vec<Sender>>`. Send failures evict the
//! tab from the list — there is no explicit unregister API, dead tabs
//! garbage-collect on the next broadcast.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Message types pushed to the browser HMR client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HmrType {
    /// Stylesheet swap — no reload, no FOUC.
    HmrCss,
    /// Partial page replacement — swap `<main>` body, preserve scroll.
    HmrHtml,
    /// Full `location.reload()`.
    Reload,
}

impl HmrType {
    /// The wire string the JS client matches on (`msg.type`).
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::HmrType;
    /// assert_eq!(HmrType::HmrCss.wire(), "hmr-css");
    /// assert_eq!(HmrType::Reload.wire(), "reload");
    /// ```
    #[must_use]
    pub const fn wire(self) -> &'static str {
        match self {
            Self::HmrCss => "hmr-css",
            Self::HmrHtml => "hmr-html",
            Self::Reload => "reload",
        }
    }
}

/// A single HMR frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HmrMessage {
    /// Frame type.
    #[serde(rename = "type")]
    pub kind: HmrType,
    /// Paths affected. For `hmr-html` these are page URLs; for
    /// `hmr-css` these are asset URLs; for `reload` this may be empty.
    pub paths: Vec<String>,
    /// Short content hash. Optional — empty string is fine.
    #[serde(default)]
    pub sha: String,
}

impl HmrMessage {
    /// Build a CSS-swap frame for one or more stylesheet paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::{HmrMessage, HmrType};
    /// let m = HmrMessage::css(vec!["x.css".into()]);
    /// assert_eq!(m.kind, HmrType::HmrCss);
    /// assert_eq!(m.paths, vec!["x.css"]);
    /// ```
    #[must_use]
    pub const fn css(paths: Vec<String>) -> Self {
        Self {
            kind: HmrType::HmrCss,
            paths,
            sha: String::new(),
        }
    }

    /// Build an HTML-partial frame for one or more page URLs.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::{HmrMessage, HmrType};
    /// let m = HmrMessage::html(vec!["/a/".into()]);
    /// assert_eq!(m.kind, HmrType::HmrHtml);
    /// ```
    #[must_use]
    pub const fn html(paths: Vec<String>) -> Self {
        Self {
            kind: HmrType::HmrHtml,
            paths,
            sha: String::new(),
        }
    }

    /// Build a full-page-reload frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::{HmrMessage, HmrType};
    /// let m = HmrMessage::reload();
    /// assert_eq!(m.kind, HmrType::Reload);
    /// assert!(m.paths.is_empty());
    /// ```
    #[must_use]
    pub const fn reload() -> Self {
        Self {
            kind: HmrType::Reload,
            paths: Vec::new(),
            sha: String::new(),
        }
    }

    /// Attach a content hash for client-side de-duplication.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::HmrMessage;
    /// let m = HmrMessage::reload().with_sha("abc123");
    /// assert_eq!(m.sha, "abc123");
    /// ```
    #[must_use]
    pub fn with_sha(mut self, sha: impl Into<String>) -> Self {
        self.sha = sha.into();
        self
    }

    /// Serialise to the JSON wire format. Infallible — the struct is
    /// always serialisable.
    ///
    /// # Panics
    ///
    /// Only if `serde_json` cannot serialise a primitive `Vec<String>`,
    /// which is impossible.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::HmrMessage;
    /// let m = HmrMessage::css(vec!["a.css".into()]);
    /// let json = m.to_json();
    /// assert!(json.contains("\"type\":\"hmr-css\""));
    /// ```
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            // Defensive: serialise a static reload as a last-ditch frame.
            r#"{"type":"reload","paths":[],"sha":""}"#.to_string()
        })
    }
}

/// Per-tab callback that emits one HMR frame.
///
/// The broadcaster keeps a vector of these and invokes each on every
/// `broadcast()`. Callbacks return `Ok(())` on a successful WebSocket
/// send and `Err(())` to be evicted (closed socket, write error).
pub type HmrSink = Box<dyn Fn(&str) -> Result<(), ()> + Send + Sync>;

/// Fan-out HMR sender shared across the dev server.
///
/// Construct one per `ssg dev` process. Each connected tab registers
/// an [`HmrSink`] via [`Self::subscribe`]; [`Self::broadcast`] pushes
/// the same frame to every live tab and evicts any sink that errored.
#[derive(Default)]
pub struct HmrBroadcaster {
    sinks: Mutex<Vec<HmrSink>>,
}

impl std::fmt::Debug for HmrBroadcaster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HmrBroadcaster")
            .field("subscribers", &self.sinks.lock().map_or(0, |v| v.len()))
            .finish()
    }
}

impl HmrBroadcaster {
    /// Construct an empty broadcaster.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::HmrBroadcaster;
    /// let b = HmrBroadcaster::new();
    /// assert_eq!(b.subscriber_count(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            sinks: Mutex::new(Vec::new()),
        }
    }

    /// Register a new tab. The sink is invoked on every subsequent
    /// [`Self::broadcast`] until it returns `Err(())`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::HmrBroadcaster;
    /// let b = HmrBroadcaster::new();
    /// b.subscribe(Box::new(|_| Ok(())));
    /// assert_eq!(b.subscriber_count(), 1);
    /// ```
    pub fn subscribe(&self, sink: HmrSink) {
        if let Ok(mut g) = self.sinks.lock() {
            g.push(sink);
        }
    }

    /// Returns the current number of subscribed tabs. Useful for tests
    /// and the dev-server status banner.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::HmrBroadcaster;
    /// let b = HmrBroadcaster::new();
    /// assert_eq!(b.subscriber_count(), 0);
    /// ```
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.sinks.lock().map_or(0, |g| g.len())
    }

    /// Push `msg` to every subscriber. Sinks that return `Err(())` are
    /// evicted in-place.
    ///
    /// Returns the number of tabs that successfully received the frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg::hmr::{HmrBroadcaster, HmrMessage};
    /// let b = HmrBroadcaster::new();
    /// b.subscribe(Box::new(|_| Ok(())));
    /// assert_eq!(b.broadcast(&HmrMessage::reload()), 1);
    /// ```
    pub fn broadcast(&self, msg: &HmrMessage) -> usize {
        let payload = msg.to_json();
        let Ok(mut sinks) = self.sinks.lock() else {
            return 0;
        };
        let mut delivered = 0usize;
        // Walk in reverse so swap_remove preserves the unvisited prefix.
        let mut i = sinks.len();
        while i > 0 {
            i -= 1;
            match (sinks[i])(&payload) {
                Ok(()) => delivered += 1,
                Err(()) => {
                    let _ = sinks.swap_remove(i);
                }
            }
        }
        delivered
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn hmr_type_wire_strings_match_spec() {
        assert_eq!(HmrType::HmrCss.wire(), "hmr-css");
        assert_eq!(HmrType::HmrHtml.wire(), "hmr-html");
        assert_eq!(HmrType::Reload.wire(), "reload");
    }

    #[test]
    fn hmr_message_css_serialises_to_kebab_type() {
        let m = HmrMessage::css(vec!["assets/style.css".into()]);
        let json = m.to_json();
        assert!(json.contains(r#""type":"hmr-css""#));
        assert!(json.contains(r#""paths":["assets/style.css"]"#));
        assert!(json.contains(r#""sha":"""#));
    }

    #[test]
    fn hmr_message_html_serialises_paths() {
        let m = HmrMessage::html(vec!["/a/".into(), "/b/".into()]);
        let json = m.to_json();
        assert!(json.contains(r#""type":"hmr-html""#));
        assert!(json.contains(r#""/a/""#));
        assert!(json.contains(r#""/b/""#));
    }

    #[test]
    fn hmr_message_reload_has_empty_paths() {
        let m = HmrMessage::reload();
        let json = m.to_json();
        assert!(json.contains(r#""type":"reload""#));
        assert!(json.contains(r#""paths":[]"#));
    }

    #[test]
    fn hmr_message_with_sha_round_trips() {
        let m = HmrMessage::html(vec!["/x/".into()]).with_sha("deadbeef");
        let json = m.to_json();
        let back: HmrMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
        assert_eq!(back.sha, "deadbeef");
    }

    #[test]
    fn hmr_message_default_sha_omittable() {
        // Without an explicit sha field on the wire, deserialise should
        // still succeed thanks to `#[serde(default)]`.
        let json = r#"{"type":"hmr-css","paths":["a.css"]}"#;
        let m: HmrMessage = serde_json::from_str(json).unwrap();
        assert_eq!(m.kind, HmrType::HmrCss);
        assert_eq!(m.sha, "");
    }

    #[test]
    fn broadcaster_new_has_zero_subscribers() {
        let b = HmrBroadcaster::new();
        assert_eq!(b.subscriber_count(), 0);
    }

    #[test]
    fn broadcaster_default_has_zero_subscribers() {
        let b = HmrBroadcaster::default();
        assert_eq!(b.subscriber_count(), 0);
    }

    #[test]
    fn subscribe_increments_count() {
        let b = HmrBroadcaster::new();
        b.subscribe(Box::new(|_| Ok(())));
        b.subscribe(Box::new(|_| Ok(())));
        assert_eq!(b.subscriber_count(), 2);
        // Deliver one frame so both registered sinks actually run.
        assert_eq!(b.broadcast(&HmrMessage::reload()), 2);
    }

    #[test]
    fn broadcast_delivers_to_every_subscriber() {
        let b = HmrBroadcaster::new();
        let calls = Arc::new(AtomicUsize::new(0));
        for _ in 0..3 {
            let calls = Arc::clone(&calls);
            b.subscribe(Box::new(move |_| {
                let _ = calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }));
        }
        let delivered = b.broadcast(&HmrMessage::reload());
        assert_eq!(delivered, 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn broadcast_evicts_failed_sinks() {
        let b = HmrBroadcaster::new();
        b.subscribe(Box::new(|_| Err(())));
        b.subscribe(Box::new(|_| Ok(())));
        b.subscribe(Box::new(|_| Err(())));

        let delivered = b.broadcast(&HmrMessage::reload());
        assert_eq!(delivered, 1);
        // The two erroring sinks should have been evicted.
        assert_eq!(b.subscriber_count(), 1);

        // Second broadcast still sees the surviving subscriber.
        let delivered2 = b.broadcast(&HmrMessage::reload());
        assert_eq!(delivered2, 1);
        assert_eq!(b.subscriber_count(), 1);
    }

    #[test]
    fn broadcast_passes_json_payload_to_sinks() {
        let b = HmrBroadcaster::new();
        let seen: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let seen_clone = Arc::clone(&seen);
        b.subscribe(Box::new(move |s| {
            *seen_clone.lock().unwrap() = Some(s.to_string());
            Ok(())
        }));

        let _ = b.broadcast(&HmrMessage::css(vec!["x.css".into()]));
        let payload = seen.lock().unwrap().clone().unwrap();
        assert!(payload.contains(r#""type":"hmr-css""#));
        assert!(payload.contains("x.css"));
    }

    #[test]
    fn broadcast_with_no_subscribers_is_noop() {
        let b = HmrBroadcaster::new();
        assert_eq!(b.broadcast(&HmrMessage::reload()), 0);
    }

    #[test]
    fn broadcaster_debug_format_includes_count() {
        let b = HmrBroadcaster::new();
        b.subscribe(Box::new(|_| Ok(())));
        let d = format!("{b:?}");
        assert!(d.contains("HmrBroadcaster"));
        assert!(d.contains('1'));
        // Run the registered sink once so the closure body executes.
        assert_eq!(b.broadcast(&HmrMessage::reload()), 1);
    }

    #[test]
    fn broadcaster_recovers_from_poisoned_sink_lock() {
        // Poison the internal sinks mutex, then exercise every lock
        // site: subscribe silently no-ops, broadcast returns 0, the
        // counters fall back to 0.
        let b = HmrBroadcaster::new();
        b.subscribe(Box::new(|_| Ok(())));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = b.sinks.lock().unwrap();
            panic!("poison sinks");
        }));

        b.subscribe(Box::new(|_| Ok(()))); // if-let Ok fails: no-op
        assert_eq!(b.subscriber_count(), 0, "poisoned lock reads as empty");
        assert_eq!(b.broadcast(&HmrMessage::reload()), 0);
        let d = format!("{b:?}");
        assert!(d.contains('0'), "Debug falls back to 0: {d}");
    }

    #[test]
    fn hmr_message_to_json_is_valid_json() {
        let m =
            HmrMessage::html(vec!["/a/".into(), "/b/".into()]).with_sha("h");
        let v: serde_json::Value = serde_json::from_str(&m.to_json()).unwrap();
        assert_eq!(v["type"], "hmr-html");
        assert_eq!(v["paths"][0], "/a/");
        assert_eq!(v["paths"][1], "/b/");
        assert_eq!(v["sha"], "h");
    }

    #[test]
    fn protocol_three_message_types_distinct() {
        let css = HmrMessage::css(vec![]).to_json();
        let html = HmrMessage::html(vec![]).to_json();
        let reload = HmrMessage::reload().to_json();
        assert!(css.contains("hmr-css"));
        assert!(html.contains("hmr-html"));
        assert!(reload.contains("reload"));
        assert!(!reload.contains("hmr-"));
    }

    #[test]
    fn hmr_type_is_copy() {
        let t = HmrType::HmrHtml;
        let _copy = t;
        assert_eq!(t, HmrType::HmrHtml);
    }
}
