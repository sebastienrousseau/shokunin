// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Schema representation shared between the proc-macro and the TS
//! emitter.
//!
//! We deliberately use a `serde_json::Value` rather than a typed
//! schemars `Schema` because:
//!
//! 1. schemars 1.x stores schemas as `serde_json::Value` internally,
//!    so we save one round-trip.
//! 2. The TS emitter walks JSON in any case — typing it would just
//!    add a translation layer.
//! 3. Equality + golden-file comparison falls out for free.

use serde::{Deserialize, Serialize};

/// A schemars draft-2020-12 JSON Schema, stored as a JSON value so
/// callers can serialise / hash / diff it without re-parsing.
pub type SchemaValue = serde_json::Value;

/// Schema bundle for a single registered RPC.
///
/// `input` is the schema of the function's argument type. `output`
/// is the schema of the success branch of its return type
/// (`Result<T, RpcError>` is unwrapped to `T`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcSchema {
    /// Registered name of the RPC.
    pub name: &'static str,
    /// JSON Schema for the input type.
    pub input: SchemaValue,
    /// JSON Schema for the success-branch output type.
    pub output: SchemaValue,
}

impl PartialEq for RpcSchema {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.input == other.input
            && self.output == other.output
    }
}

/// Build a JSON Schema for an arbitrary `T: JsonSchema`.
///
/// Returned as a `serde_json::Value` for the reasons spelled out in
/// the module docs.
#[must_use]
pub fn schema_for<T: schemars::JsonSchema>() -> SchemaValue {
    let generator = schemars::SchemaGenerator::default();
    let schema = generator.into_root_schema_for::<T>();
    serde_json::to_value(schema).unwrap_or(serde_json::Value::Null)
}

/// Build a JSON Schema for the success arm of a
/// `Result<T, RpcError>`-shaped type.
///
/// The macro can't easily strip the `Result<…, RpcError>` wrapper at
/// the type level (proc-macros work on tokens, not on resolved
/// types), so we lean on a trait-based unwrapping helper:
///
/// * For `Result<T, _>`, the success schema is `T`'s schema.
/// * For everything else, we fall back to that type's own schema —
///   which lets us write integration tests that pass non-Result
///   return types.
#[must_use]
pub fn schema_for_result<T: ResultLikeSchema>() -> SchemaValue {
    T::success_schema()
}

/// Trait that lets us produce the "success" schema for any type the
/// proc-macro might see in a return position.
pub trait ResultLikeSchema {
    /// Returns the schema of the success arm.
    fn success_schema() -> SchemaValue;
}

impl<T, E> ResultLikeSchema for Result<T, E>
where
    T: schemars::JsonSchema,
{
    fn success_schema() -> SchemaValue {
        schema_for::<T>()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[allow(dead_code)]
    struct Greet {
        who: String,
    }

    #[test]
    fn schema_for_struct_has_properties() {
        let schema = schema_for::<Greet>();
        // schemars 1.x emits a `properties` map at the top level.
        let props = schema.get("properties").expect("properties present");
        assert!(props.get("who").is_some());
    }

    #[test]
    fn schema_for_result_unwraps_ok_branch() {
        let schema = schema_for_result::<Result<Greet, crate::RpcError>>();
        let props = schema.get("properties").expect("properties present");
        assert!(props.get("who").is_some());
    }

    #[test]
    fn rpc_schema_equality_is_field_wise() {
        let a = RpcSchema {
            name: "x",
            input: schema_for::<Greet>(),
            output: schema_for::<Greet>(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn rpc_schema_serialises_to_json() {
        let s = RpcSchema {
            name: "x",
            input: serde_json::json!({"type": "string"}),
            output: serde_json::json!({"type": "string"}),
        };
        let txt = serde_json::to_string(&s).unwrap();
        assert!(txt.contains("\"name\":\"x\""));
    }
}
