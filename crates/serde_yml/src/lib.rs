// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A safe, local YAML serialization/deserialization library
//! compatible with the `serde_yml` API.
//!
//! This crate provides a drop-in replacement for
//! `serde_yml` using only safe Rust, eliminating the
//! dependency on the unsafe `libyml` C library.

#![forbid(unsafe_code)]

pub mod de;
pub mod error;
pub mod mapping;
pub mod number;
pub mod ser;
pub mod value;

pub use de::{from_reader, from_slice, from_str, Deserializer};
pub use error::{Error, Location, Result};
pub use mapping::Mapping;
pub use number::Number;
pub use ser::{to_string, to_writer, Serializer, State};
pub use value::tagged::{Tag, TaggedValue};
pub use value::{from_value, to_value, Sequence, Value};
