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
///
/// # Examples
///
/// ```
/// use ssg_rpc::schema::{schema_for, SchemaValue};
///
/// // `SchemaValue` is just a type alias for `serde_json::Value`, so
/// // ordinary `serde_json` helpers apply.
/// let s: SchemaValue = schema_for::<String>();
/// assert!(s.is_object());
/// assert!(s.get("type").is_some());
/// ```
pub type SchemaValue = serde_json::Value;

/// Schema bundle for a single registered RPC.
///
/// `input` is the schema of the function's argument type. `output`
/// is the schema of the success branch of its return type
/// (`Result<T, RpcError>` is unwrapped to `T`).
///
/// # Examples
///
/// ```
/// use ssg_rpc::schema::{schema_for, RpcSchema};
///
/// let s = RpcSchema {
///     name: "greet",
///     input: schema_for::<String>(),
///     output: schema_for::<String>(),
/// };
/// assert_eq!(s.name, "greet");
///
/// // Field-wise `PartialEq` falls out of the JSON-backed schemas.
/// let s2 = s.clone();
/// assert_eq!(s, s2);
/// ```
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
///
/// # Examples
///
/// ```
/// use ssg_rpc::schema::schema_for;
///
/// let schema = schema_for::<String>();
/// // schemars renders `String` with a `"type"` keyword of `"string"`
/// // (sometimes wrapped in a single-entry array).
/// let t = schema.get("type").expect("type key present");
/// let is_string = match t {
///     serde_json::Value::String(s) => s == "string",
///     serde_json::Value::Array(arr) => {
///         arr.iter().any(|v| v.as_str() == Some("string"))
///     }
///     _ => false,
/// };
/// assert!(is_string, "expected string type, got {schema}");
/// ```
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
///
/// # Examples
///
/// ```
/// use ssg_rpc::schema::{schema_for, schema_for_result};
/// use ssg_rpc::RpcError;
///
/// // The Ok-branch schema of `Result<String, RpcError>` equals the
/// // schema of plain `String`.
/// let unwrapped = schema_for_result::<Result<String, RpcError>>();
/// assert_eq!(unwrapped, schema_for::<String>());
/// ```
#[must_use]
pub fn schema_for_result<T: ResultLikeSchema>() -> SchemaValue {
    T::success_schema()
}

/// Trait that lets us produce the "success" schema for any type the
/// proc-macro might see in a return position.
///
/// # Examples
///
/// ```
/// use ssg_rpc::schema::{schema_for, ResultLikeSchema};
/// use ssg_rpc::RpcError;
///
/// // The blanket impl for `Result<T, E>` returns T's schema.
/// let s = <Result<String, RpcError> as ResultLikeSchema>::success_schema();
/// assert_eq!(s, schema_for::<String>());
/// ```
pub trait ResultLikeSchema {
    /// Returns the schema of the success arm.
    ///
    /// # Examples
    ///
    /// ```
    /// use ssg_rpc::schema::{schema_for, ResultLikeSchema};
    /// use ssg_rpc::RpcError;
    ///
    /// let s = <Result<u32, RpcError>>::success_schema();
    /// // `u32` is a numeric (integer) type — schemars emits some
    /// // `"type"` value, never null.
    /// assert!(!s.is_null());
    /// assert!(s.get("type").is_some());
    /// ```
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
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

    #[test]
    fn rpc_schema_equality_distinguishes_name() {
        let a = RpcSchema {
            name: "a",
            input: schema_for::<Greet>(),
            output: schema_for::<Greet>(),
        };
        let b = RpcSchema {
            name: "b",
            input: schema_for::<Greet>(),
            output: schema_for::<Greet>(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn rpc_schema_equality_distinguishes_input() {
        let a = RpcSchema {
            name: "x",
            input: schema_for::<Greet>(),
            output: schema_for::<String>(),
        };
        let b = RpcSchema {
            name: "x",
            input: schema_for::<String>(),
            output: schema_for::<String>(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn rpc_schema_clone_preserves_fields() {
        let a = RpcSchema {
            name: "clone_me",
            input: serde_json::json!({"type": "string"}),
            output: serde_json::json!({"type": "integer"}),
        };
        let cloned = a.clone();
        assert_eq!(cloned.name, "clone_me");
        assert_eq!(cloned.input, serde_json::json!({"type": "string"}));
        assert_eq!(cloned.output, serde_json::json!({"type": "integer"}));
    }

    #[test]
    fn schema_for_primitive_string_emits_type_string() {
        let schema = schema_for::<String>();
        // schemars emits `"type": "string"` (possibly inside a
        // single-entry array) for `String`.
        let t = schema.get("type").expect("type key present");
        let matches = match t {
            serde_json::Value::String(s) => s == "string",
            serde_json::Value::Array(arr) => {
                arr.iter().any(|v| v.as_str() == Some("string"))
            }
            _ => false,
        };
        assert!(matches, "expected string type, got {schema}");
    }

    #[test]
    fn schema_for_result_with_primitive_ok() {
        let schema = schema_for_result::<Result<u32, crate::RpcError>>();
        // schemars emits an integer type, possibly with constraints
        // like `format` or `minimum` — just verify it isn't null.
        assert!(!schema.is_null());
    }

    #[test]
    fn schema_serialises_name_field_exactly() {
        // Round-trip via Deserialize requires `'static` borrow lifetime
        // because the `name` field is `&'static str` — so we only
        // assert the serialised JSON shape here.
        let original = RpcSchema {
            name: "rt",
            input: schema_for::<Greet>(),
            output: schema_for::<Greet>(),
        };
        let json = serde_json::to_string(&original).unwrap();
        assert!(json.contains("\"name\":\"rt\""));
        assert!(json.contains("\"input\""));
        assert!(json.contains("\"output\""));
    }

    #[test]
    fn result_like_schema_unwraps_via_trait() {
        // Direct trait call path (the public `schema_for_result`
        // delegates here).
        let s = <Result<Greet, crate::RpcError> as ResultLikeSchema>::success_schema();
        assert!(s.get("properties").is_some());
    }
}
