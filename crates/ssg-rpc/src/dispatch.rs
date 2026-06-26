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
pub type DispatchFn = fn(&str) -> Result<String, RpcError>;

/// Thunk producing the schema for a single RPC.
///
/// Held as a function pointer rather than a `&'static RpcSchema` so
/// the schema can be built lazily — schemars allocations would
/// otherwise happen at program start for every registered RPC,
/// even if the user never runs the `.d.ts` emitter.
pub type SchemaFn = fn() -> RpcSchema;

/// Static descriptor produced by the `#[ssg_rpc]` macro.
///
/// `Debug` is derived manually to silence Clippy on function-pointer
/// fields — the address isn't useful but at least the name is.
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
#[derive(Clone, Copy, Debug)]
pub struct RpcDescriptorRef(pub &'static RpcDescriptor);

inventory::collect!(RpcDescriptorRef);

/// Cold-path lookup: returns every descriptor registered into the
/// inventory. Used by the schema emitter and by tests; the hot path
/// goes through [`dispatch`] which caches into a map.
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
#[must_use]
pub fn find(name: &str) -> Option<&'static RpcDescriptor> {
    registry().get(name).copied()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
