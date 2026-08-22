// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Dispatch registry — name → trampoline lookup.
//!
//! Functions annotated `#[ssg_rpc]` register themselves into the
//! [`inventory`] collector at link time. Lookups walk the iterator
//! once on cold path; for hot dispatch we cache into a `HashMap`
//! the first time `dispatch` is called.
//!
//! The cached map is read-once (`OnceLock`) so we don't need any
//! locking on the hot path — once initialised the map is immutable
//! shared state, which is fine because the inventory is fixed at
//! load time.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::schema::RpcSchema;
use crate::RpcError;

/// Type-erased dispatcher signature emitted by the proc-macro.
///
/// Takes the raw JSON request body, returns either the JSON response
/// body or an [`RpcError`].
///
/// # Examples
///
/// ```
/// use ssg_rpc::dispatch::DispatchFn;
/// use ssg_rpc::RpcError;
///
/// // A trivial dispatch function that echoes its JSON input.
/// let echo: DispatchFn = |payload| {
///     let v: serde_json::Value = serde_json::from_str(payload)
///         .map_err(|e| RpcError::BadRequest(e.to_string()))?;
///     serde_json::to_string(&v)
///         .map_err(|e| RpcError::Internal(e.to_string()))
/// };
///
/// assert_eq!(echo("\"hi\"").unwrap(), "\"hi\"");
/// assert!(matches!(echo("{not json").unwrap_err(),
///     RpcError::BadRequest(_)));
/// ```
pub type DispatchFn = fn(&str) -> Result<String, RpcError>;

/// Thunk producing the schema for a single RPC.
///
/// Held as a function pointer rather than a `&'static RpcSchema` so
/// the schema can be built lazily — schemars allocations would
/// otherwise happen at program start for every registered RPC,
/// even if the user never runs the `.d.ts` emitter.
///
/// # Examples
///
/// ```
/// use ssg_rpc::dispatch::SchemaFn;
/// use ssg_rpc::schema::{schema_for, RpcSchema};
///
/// let producer: SchemaFn = || RpcSchema {
///     name: "ping",
///     input: schema_for::<String>(),
///     output: schema_for::<String>(),
/// };
///
/// let s = producer();
/// assert_eq!(s.name, "ping");
/// ```
pub type SchemaFn = fn() -> RpcSchema;

/// Static descriptor produced by the `#[ssg_rpc]` macro.
///
/// `Debug` is derived manually to silence Clippy on function-pointer
/// fields — the address isn't useful but at least the name is.
///
/// # Examples
///
/// ```
/// use ssg_rpc::dispatch::RpcDescriptor;
/// use ssg_rpc::schema::{schema_for, RpcSchema};
/// use ssg_rpc::RpcError;
///
/// fn dispatch(payload: &str) -> Result<String, RpcError> {
///     Ok(payload.to_string())
/// }
///
/// fn schema() -> RpcSchema {
///     RpcSchema {
///         name: "id",
///         input: schema_for::<String>(),
///         output: schema_for::<String>(),
///     }
/// }
///
/// let desc = RpcDescriptor { name: "id", dispatch, schema };
/// assert_eq!(desc.name, "id");
/// assert_eq!((desc.dispatch)("\"x\"").unwrap(), "\"x\"");
/// // Debug renders the name without the function-pointer address.
/// assert!(format!("{desc:?}").contains("\"id\""));
/// ```
#[derive(Clone, Copy)]
pub struct RpcDescriptor {
    /// Public name of the RPC. Routed at `POST /__rpc/<name>`.
    pub name: &'static str,
    /// JSON trampoline emitted by the proc-macro.
    pub dispatch: DispatchFn,
    /// Schema producer.
    pub schema: SchemaFn,
}

impl std::fmt::Debug for RpcDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcDescriptor")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// Newtype wrapper holding a registered descriptor reference.
///
/// `inventory::submit!` needs a `Sync + 'static` owned type — a
/// `&'static RpcDescriptor` satisfies both. The wrapper exists only
/// to give `inventory::collect!` a single concrete type per
/// collection, which is how the macro avoids one-collection-per-RPC.
///
/// # Examples
///
/// The proc-macro normally constructs and submits these for you. The
/// example below shows the manual shape (ignored because
/// `inventory::submit!` requires a `static` at module scope, which
/// rustdoc can't express inside a fenced block):
///
/// ```ignore
/// use ssg_rpc::dispatch::{RpcDescriptor, RpcDescriptorRef};
///
/// static MY_DESC: RpcDescriptor = RpcDescriptor {
///     name: "my_fn",
///     dispatch: my_dispatch,
///     schema: my_schema,
/// };
/// inventory::submit! { RpcDescriptorRef(&MY_DESC) }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RpcDescriptorRef(pub &'static RpcDescriptor);

inventory::collect!(RpcDescriptorRef);

/// Cold-path lookup: returns every descriptor registered into the
/// inventory. Used by the schema emitter and by tests; the hot path
/// goes through [`dispatch`] which caches into a map.
///
/// # Examples
///
/// ```
/// // Iterating the inventory is always safe — the iterator yields
/// // every descriptor the proc-macro has registered at link time.
/// // In a doctest binary (no `#[ssg_rpc]` functions linked in) the
/// // iterator may be empty; either way, calling `count()` cannot panic.
/// let count = ssg_rpc::dispatch::iter_descriptors().count();
/// assert_eq!(count, ssg_rpc::dispatch::iter_descriptors().count());
/// ```
pub fn iter_descriptors() -> impl Iterator<Item = &'static RpcDescriptor> {
    inventory::iter::<RpcDescriptorRef>().map(|r| r.0)
}

fn registry() -> &'static HashMap<&'static str, &'static RpcDescriptor> {
    static REGISTRY: OnceLock<HashMap<&'static str, &'static RpcDescriptor>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut map: HashMap<&'static str, &'static RpcDescriptor> =
            HashMap::new();
        for desc in iter_descriptors() {
            // First registration wins. The macro-generated names are
            // deterministic so duplicates would imply a real bug in
            // the user's code — we surface that via a debug_assert
            // rather than a runtime panic on the hot path.
            debug_assert!(
                !map.contains_key(desc.name),
                "duplicate ssg_rpc registration: {}",
                desc.name
            );
            let _ = map.insert(desc.name, desc);
        }
        map
    })
}

/// Returns the names of every registered RPC. Sorted for stable
/// output across builds; callers that need raw order can iterate
/// [`iter_descriptors`] instead.
///
/// **Do not expose this over the wire.** AC3 of issue #548 forbids
/// leaking registered names so an unauthenticated caller cannot
/// enumerate the surface.
///
/// # Examples
///
/// ```
/// let names = ssg_rpc::dispatch::registered_names();
/// // Sorted, deterministic output: calling twice yields the same
/// // ordering. (In a doctest binary the list may be empty, since
/// // no `#[ssg_rpc]` functions are linked in.)
/// let mut sorted = names.clone();
/// sorted.sort_unstable();
/// assert_eq!(names, sorted);
/// assert_eq!(names, ssg_rpc::dispatch::registered_names());
/// ```
pub fn registered_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = registry().keys().copied().collect();
    names.sort_unstable();
    names
}

/// Dispatch a JSON RPC call.
///
/// Returns:
/// * `Ok(json_string)` when the function returned `Ok(_)`.
/// * `Err(RpcError::NotFound)` when `name` is not registered. The
///   dispatcher uses `NotFound` (404) so the surface is opaque —
///   the registry list is not exposed.
/// * `Err(RpcError::BadRequest(_))` when `payload` doesn't deserialise
///   into the function's input type.
/// * `Err(RpcError::Internal(_))` when the response can't be
///   serialised.
/// * Any `RpcError` variant the function itself returned.
///
/// # Examples
///
/// ```
/// use ssg_rpc::{dispatch::dispatch, RpcError};
///
/// // Unknown names map to NotFound — the dispatcher never leaks
/// // whether a name was a typo or genuinely absent.
/// let err = dispatch("definitely_not_real", "{}").unwrap_err();
/// assert!(matches!(err, RpcError::NotFound));
/// assert_eq!(err.status_code(), 404);
/// ```
///
/// A round-trip against a real, registered RPC (the `#[ssg_rpc]`
/// macro generates the descriptor and submits it to the inventory):
///
/// ```ignore
/// use ssg_rpc::dispatch::dispatch;
///
/// // Assuming `#[ssg_rpc] fn echo(s: String) -> Result<String, _> { Ok(s) }`
/// let out = dispatch("echo", "\"hello\"").unwrap();
/// assert_eq!(out, "\"hello\"");
/// ```
pub fn dispatch(name: &str, payload: &str) -> Result<String, RpcError> {
    let Some(desc) = registry().get(name) else {
        return Err(RpcError::NotFound);
    };
    (desc.dispatch)(payload)
}

/// Returns the descriptor for a given RPC name, if registered.
///
/// Mostly used by the TS emitter and tests; the hot dispatch path
/// uses [`dispatch`] directly.
///
/// # Examples
///
/// ```
/// // Lookup of an unregistered name yields `None` — never a panic.
/// assert!(ssg_rpc::dispatch::find("nope").is_none());
/// ```
///
/// Once an RPC is registered via `#[ssg_rpc]`, looking it up by
/// name returns the descriptor:
///
/// ```ignore
/// let desc = ssg_rpc::dispatch::find("echo").expect("echo registered");
/// assert_eq!(desc.name, "echo");
/// ```
#[must_use]
pub fn find(name: &str) -> Option<&'static RpcDescriptor> {
    registry().get(name).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{schema_for, schema_for_result, RpcSchema};

    // Hand-rolled descriptor (the proc-macro tests live in the
    // top-level `tests/` dir; here we only exercise the registry
    // mechanics).
    fn echo_dispatch(payload: &str) -> Result<String, RpcError> {
        let value: serde_json::Value = serde_json::from_str(payload)
            .map_err(|e| RpcError::BadRequest(e.to_string()))?;
        serde_json::to_string(&value)
            .map_err(|e| RpcError::Internal(format!("ser: {e}")))
    }

    fn echo_schema() -> RpcSchema {
        RpcSchema {
            name: "echo",
            input: schema_for::<String>(),
            output: schema_for_result::<Result<String, RpcError>>(),
        }
    }

    static ECHO: RpcDescriptor = RpcDescriptor {
        name: "echo",
        dispatch: echo_dispatch,
        schema: echo_schema,
    };

    inventory::submit! { RpcDescriptorRef(&ECHO) }

    #[test]
    fn registered_names_contains_echo() {
        let names = registered_names();
        assert!(names.contains(&"echo"), "{names:?}");
    }

    #[test]
    fn dispatch_echo_round_trips() {
        let out = dispatch("echo", "\"hello\"").unwrap();
        assert_eq!(out, "\"hello\"");
    }

    #[test]
    fn dispatch_unknown_returns_not_found() {
        let err = dispatch("definitely_not_real", "{}").unwrap_err();
        assert!(matches!(err, RpcError::NotFound));
    }

    #[test]
    fn dispatch_bad_payload_returns_bad_request() {
        let err = dispatch("echo", "{not json").unwrap_err();
        assert!(matches!(err, RpcError::BadRequest(_)));
    }

    #[test]
    fn find_returns_descriptor() {
        let desc = find("echo").unwrap();
        assert_eq!(desc.name, "echo");
    }

    #[test]
    fn find_returns_none_for_unknown() {
        assert!(find("nope").is_none());
    }

    #[test]
    fn iter_descriptors_yields_at_least_one() {
        assert!(iter_descriptors().count() >= 1);
    }
}
