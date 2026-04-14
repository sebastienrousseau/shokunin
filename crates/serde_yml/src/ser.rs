// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! YAML serialization: `to_string` and `to_writer`.

use crate::{
    error::Error,
    value::{self, Value},
};
use serde::Serialize;
use std::io::Write;

type Result<T> = std::result::Result<T, Error>;

/// Serialize the given value as a YAML string.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let v = value.serialize(value::ValueSerializer)?;
    let mut out = String::new();
    emit_value(&v, &mut out, 0, false);
    Ok(out)
}

/// Serialize the given value as YAML into a writer.
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    let s = to_string(value)?;
    writer
        .write_all(s.as_bytes())
        .map_err(|e| Error::msg(e.to_string()))
}

/// The state of the YAML serializer (for API compat).
#[derive(Debug)]
pub enum State {
    /// Nothing in particular.
    NothingInParticular,
}

/// A YAML serializer (for API compat).
#[derive(Debug)]
pub struct Serializer {
    _private: (),
}

fn emit_value(
    v: &Value,
    out: &mut String,
    indent: usize,
    inline: bool,
) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => {
            out.push_str(if *b { "true" } else { "false" });
        }
        Value::Number(n) => {
            out.push_str(&n.to_string());
        }
        Value::String(s) => {
            emit_string(s, out);
        }
        Value::Sequence(seq) => {
            if seq.is_empty() {
                out.push_str("[]");
            } else if inline {
                // For inline, use flow style
                emit_flow_sequence(seq, out);
            } else {
                emit_block_sequence(seq, out, indent);
            }
        }
        Value::Mapping(m) => {
            if m.is_empty() {
                out.push_str("{}");
            } else if inline {
                emit_flow_mapping(m, out);
            } else {
                emit_block_mapping(m, out, indent);
            }
        }
        Value::Tagged(t) => {
            out.push_str(&format!("{} ", t.tag));
            emit_value(&t.value, out, indent, inline);
        }
    }
}

fn emit_string(s: &str, out: &mut String) {
    if s.is_empty() {
        out.push_str("''");
        return;
    }
    // Check if the string needs quoting
    if needs_quoting(s) {
        out.push('\'');
        // Escape single quotes by doubling
        for ch in s.chars() {
            if ch == '\'' {
                out.push_str("''");
            } else {
                out.push(ch);
            }
        }
        out.push('\'');
    } else {
        out.push_str(s);
    }
}

fn needs_quoting(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    // Values that would be interpreted as non-string
    match s {
        "null" | "Null" | "NULL" | "~" | "true"
        | "True" | "TRUE" | "false" | "False" | "FALSE"
        | ".nan" | ".NaN" | ".NAN" | ".inf" | ".Inf"
        | ".INF" | "-.inf" | "-.Inf" | "-.INF" => {
            return true
        }
        _ => {}
    }
    let first = s.as_bytes()[0];
    // Starts with special char
    if matches!(
        first,
        b'{'
            | b'}'
            | b'['
            | b']'
            | b','
            | b'&'
            | b'*'
            | b'!'
            | b'|'
            | b'>'
            | b'%'
            | b'@'
            | b'`'
            | b'\''
            | b'"'
    ) {
        return true;
    }
    // Contains problematic chars
    if s.contains(": ")
        || s.contains(" #")
        || s.contains('\n')
        || s.contains('\r')
        || s.starts_with("- ")
        || s.starts_with("? ")
    {
        return true;
    }
    // Looks like a number
    if s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok()
    {
        return true;
    }
    false
}

fn emit_block_sequence(
    seq: &[Value],
    out: &mut String,
    indent: usize,
) {
    for (i, item) in seq.iter().enumerate() {
        if i > 0 || !out.is_empty() {
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
        emit_indent(out, indent);
        out.push_str("- ");
        match item {
            Value::Mapping(m) if !m.is_empty() => {
                // First entry inline after "-", rest
                // indented
                let mut first = true;
                for (k, v) in m {
                    if first {
                        first = false;
                    } else {
                        out.push('\n');
                        emit_indent(out, indent + 2);
                    }
                    emit_value(k, out, indent + 2, true);
                    out.push_str(": ");
                    if is_compound(v) {
                        out.push('\n');
                        emit_value(
                            v,
                            out,
                            indent + 4,
                            false,
                        );
                    } else {
                        emit_value(
                            v,
                            out,
                            indent + 4,
                            true,
                        );
                    }
                }
            }
            Value::Sequence(s) if !s.is_empty() => {
                out.push('\n');
                emit_block_sequence(
                    s,
                    out,
                    indent + 2,
                );
            }
            _ => {
                emit_value(
                    item,
                    out,
                    indent + 2,
                    true,
                );
            }
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
}

fn emit_block_mapping(
    m: &crate::mapping::Mapping,
    out: &mut String,
    indent: usize,
) {
    for (i, (k, v)) in m.iter().enumerate() {
        if i > 0 || (!out.is_empty() && !out.ends_with('\n'))
        {
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
        emit_indent(out, indent);
        emit_value(k, out, indent, true);
        out.push_str(": ");
        if is_compound(v) {
            out.push('\n');
            emit_value(v, out, indent + 2, false);
        } else {
            emit_value(v, out, indent + 2, true);
            out.push('\n');
        }
    }
}

fn emit_flow_sequence(seq: &[Value], out: &mut String) {
    out.push('[');
    for (i, item) in seq.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        emit_value(item, out, 0, true);
    }
    out.push(']');
}

fn emit_flow_mapping(
    m: &crate::mapping::Mapping,
    out: &mut String,
) {
    out.push('{');
    for (i, (k, v)) in m.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        emit_value(k, out, 0, true);
        out.push_str(": ");
        emit_value(v, out, 0, true);
    }
    out.push('}');
}

fn emit_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn is_compound(v: &Value) -> bool {
    matches!(
        v,
        Value::Mapping(m) if !m.is_empty()
    ) || matches!(
        v,
        Value::Sequence(s) if !s.is_empty()
    )
}
