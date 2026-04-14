// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// A representation of YAML's `!Tag` syntax.
pub mod tagged;

use crate::{
    error::Error,
    mapping::{Index, Mapping},
    number::{self, Number},
};
use serde::{
    de::{
        self, Deserialize, DeserializeOwned, IntoDeserializer,
        MapAccess, SeqAccess, VariantAccess as _, Visitor,
    },
    forward_to_deserialize_any, Serialize, Serializer,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    mem,
};

pub use self::tagged::{Tag, TaggedValue};

/// Represents any valid YAML value.
#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value {
    /// A YAML null value.
    Null,
    /// A YAML boolean.
    Bool(bool),
    /// A YAML number.
    Number(Number),
    /// A YAML string.
    String(String),
    /// A YAML sequence.
    Sequence(Sequence),
    /// A YAML mapping.
    Mapping(Mapping),
    /// A tagged YAML value.
    Tagged(Box<TaggedValue>),
}

/// A YAML sequence.
pub type Sequence = Vec<Value>;

impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}

/// Converts a serializable value into a `Value`.
pub fn to_value<T>(value: T) -> Result<Value, Error>
where
    T: Serialize,
{
    value.serialize(ValueSerializer)
}

/// Interpret a `Value` as an instance of type `T`.
pub fn from_value<T>(value: Value) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    Deserialize::deserialize(value)
}

impl Value {
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        match self.untag_ref() {
            Value::Mapping(m) => index.index_into(m),
            Value::Sequence(_) => None,
            _ => None,
        }
    }

    pub fn get_mut<I: Index>(
        &mut self,
        index: I,
    ) -> Option<&mut Value> {
        match self.untag_mut() {
            Value::Mapping(m) => index.index_into_mut(m),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self.untag_ref(), Value::Null)
    }

    pub fn as_null(&self) -> Option<()> {
        match self.untag_ref() {
            Value::Null => Some(()),
            _ => None,
        }
    }

    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.untag_ref() {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self.untag_ref(), Value::Number(_))
    }

    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self.untag_ref() {
            Value::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    pub fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self.untag_ref() {
            Value::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    pub fn is_f64(&self) -> bool {
        match self.untag_ref() {
            Value::Number(n) => n.is_f64(),
            _ => false,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.untag_ref() {
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn as_str(&self) -> Option<&str> {
        match self.untag_ref() {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_sequence(&self) -> bool {
        self.as_sequence().is_some()
    }

    pub fn as_sequence(&self) -> Option<&Sequence> {
        match self.untag_ref() {
            Value::Sequence(seq) => Some(seq),
            _ => None,
        }
    }

    pub fn as_sequence_mut(
        &mut self,
    ) -> Option<&mut Sequence> {
        match self.untag_mut() {
            Value::Sequence(seq) => Some(seq),
            _ => None,
        }
    }

    pub fn is_mapping(&self) -> bool {
        self.as_mapping().is_some()
    }

    pub fn as_mapping(&self) -> Option<&Mapping> {
        match self.untag_ref() {
            Value::Mapping(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_mapping_mut(
        &mut self,
    ) -> Option<&mut Mapping> {
        match self.untag_mut() {
            Value::Mapping(map) => Some(map),
            _ => None,
        }
    }

    /// Merge `<<` keys into the surrounding mapping.
    pub fn apply_merge(&mut self) -> Result<(), Error> {
        let mut stack = vec![self as &mut Value];
        while let Some(node) = stack.pop() {
            match node {
                Value::Mapping(mapping) => {
                    if let Some(merge) =
                        mapping.remove("<<")
                    {
                        match merge {
                            Value::Mapping(m) => {
                                for (k, v) in m {
                                    mapping
                                        .entry(k)
                                        .or_insert(v);
                                }
                            }
                            Value::Sequence(seq) => {
                                for item in seq {
                                    if let Value::Mapping(m) = item {
                                        for (k, v) in m {
                                            mapping.entry(k).or_insert(v);
                                        }
                                    } else {
                                        return Err(Error::msg("expected mapping in merge element"));
                                    }
                                }
                            }
                            _ => {
                                return Err(Error::msg(
                                    "expected mapping for merge",
                                ));
                            }
                        }
                    }
                    stack.extend(mapping.values_mut());
                }
                Value::Sequence(seq) => {
                    stack.extend(seq.iter_mut());
                }
                Value::Tagged(tagged) => {
                    stack.push(&mut tagged.value);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn untag_ref(&self) -> &Self {
        let mut cur = self;
        while let Value::Tagged(tagged) = cur {
            cur = &tagged.value;
        }
        cur
    }

    pub(crate) fn untag_mut(&mut self) -> &mut Self {
        let mut cur = self;
        while let Value::Tagged(tagged) = cur {
            cur = &mut tagged.value;
        }
        cur
    }

    pub(crate) fn unexpected(
        &self,
    ) -> serde::de::Unexpected<'_> {
        match self {
            Value::Null => serde::de::Unexpected::Unit,
            Value::Bool(b) => serde::de::Unexpected::Bool(*b),
            Value::Number(n) => number::unexpected(n),
            Value::String(s) => {
                serde::de::Unexpected::Str(s)
            }
            Value::Sequence(_) => {
                serde::de::Unexpected::Seq
            }
            Value::Mapping(_) => {
                serde::de::Unexpected::Map
            }
            Value::Tagged(t) => t.value.unexpected(),
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Value::Null => {}
            Value::Bool(v) => v.hash(state),
            Value::Number(v) => v.hash(state),
            Value::String(v) => v.hash(state),
            Value::Sequence(v) => v.hash(state),
            Value::Mapping(v) => v.hash(state),
            Value::Tagged(v) => v.hash(state),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => f.write_str("Null"),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Number(n) => write!(f, "{:?}", n),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::Sequence(s) => {
                f.debug_list().entries(s).finish()
            }
            Value::Mapping(m) => Debug::fmt(m, f),
            Value::Tagged(t) => write!(
                f,
                "Tagged({} {:?})",
                t.tag, t.value
            ),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(b) => Display::fmt(b, f),
            Value::Number(n) => Display::fmt(n, f),
            Value::String(s) => f.write_str(s),
            Value::Sequence(_) => f.write_str("[...]"),
            Value::Mapping(_) => f.write_str("{...}"),
            Value::Tagged(t) => {
                write!(f, "{} {}", t.tag, t.value)
            }
        }
    }
}

impl IntoDeserializer<'_, Error> for Value {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

// ---- From impls ----

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_owned())
    }
}

impl From<Number> for Value {
    fn from(n: Number) -> Self {
        Value::Number(n)
    }
}

macro_rules! from_integer {
    ($($ty:ty)*) => {
        $(
            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    Value::Number(Number::from(n))
                }
            }
        )*
    };
}

from_integer!(i8 i16 i32 i64 isize u8 u16 u32 u64 usize);

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Number(Number::from(f))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(Number::from(f))
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Sequence(v.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

// ---- PartialEq with primitives ----

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        self.as_str().map_or(false, |s| s == other)
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().map_or(false, |s| s == *other)
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        self.as_str().map_or(false, |s| s == other)
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        self.as_bool().map_or(false, |b| b == *other)
    }
}

macro_rules! partialeq_integer {
    ($($ty:ty)*) => {
        $(
            impl PartialEq<$ty> for Value {
                fn eq(&self, other: &$ty) -> bool {
                    match self.untag_ref() {
                        Value::Number(n) => {
                            n.as_i64().map_or(false, |v| v == *other as i64)
                        }
                        _ => false,
                    }
                }
            }
        )*
    };
}

partialeq_integer!(i8 i16 i32 i64 u8 u16 u32);

impl PartialEq<u64> for Value {
    fn eq(&self, other: &u64) -> bool {
        match self.untag_ref() {
            Value::Number(n) => {
                n.as_u64().map_or(false, |v| v == *other)
            }
            _ => false,
        }
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match self.untag_ref() {
            Value::Number(n) => {
                n.as_f64().map_or(false, |v| v == *other)
            }
            _ => false,
        }
    }
}

// ---- total_cmp (for mapping ordering) ----

pub(crate) fn total_cmp(a: &Value, b: &Value) -> Ordering {
    match (a, b) {
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Less,
        (_, Value::Null) => Ordering::Greater,
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        (Value::Bool(_), _) => Ordering::Less,
        (_, Value::Bool(_)) => Ordering::Greater,
        (Value::Number(a), Value::Number(b)) => {
            a.total_cmp(b)
        }
        (Value::Number(_), _) => Ordering::Less,
        (_, Value::Number(_)) => Ordering::Greater,
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::String(_), _) => Ordering::Less,
        (_, Value::String(_)) => Ordering::Greater,
        (Value::Sequence(a), Value::Sequence(b)) => {
            a.partial_cmp(b).unwrap_or(Ordering::Equal)
        }
        (Value::Sequence(_), _) => Ordering::Less,
        (_, Value::Sequence(_)) => Ordering::Greater,
        (Value::Mapping(a), Value::Mapping(b)) => {
            a.partial_cmp(b).unwrap_or(Ordering::Equal)
        }
        (Value::Mapping(_), _) => Ordering::Less,
        (_, Value::Mapping(_)) => Ordering::Greater,
        (Value::Tagged(a), Value::Tagged(b)) => a
            .tag
            .cmp(&b.tag)
            .then_with(|| total_cmp(&a.value, &b.value)),
    }
}

// ---- Serialize ----

impl Serialize for Value {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(n) => n.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            Value::Sequence(seq) => seq.serialize(serializer),
            Value::Mapping(m) => m.serialize(serializer),
            Value::Tagged(t) => t.serialize(serializer),
        }
    }
}

// ---- Deserialize ----

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(
                &self,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                f.write_str("any YAML value")
            }

            fn visit_bool<E>(
                self,
                v: bool,
            ) -> Result<Value, E> {
                Ok(Value::Bool(v))
            }

            fn visit_i64<E>(
                self,
                v: i64,
            ) -> Result<Value, E> {
                Ok(Value::Number(v.into()))
            }

            fn visit_u64<E>(
                self,
                v: u64,
            ) -> Result<Value, E> {
                Ok(Value::Number(v.into()))
            }

            fn visit_f64<E>(
                self,
                v: f64,
            ) -> Result<Value, E> {
                Ok(Value::Number(v.into()))
            }

            fn visit_str<E>(
                self,
                v: &str,
            ) -> Result<Value, E> {
                Ok(Value::String(v.to_owned()))
            }

            fn visit_string<E>(
                self,
                v: String,
            ) -> Result<Value, E> {
                Ok(Value::String(v))
            }

            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_some<D>(
                self,
                deserializer: D,
            ) -> Result<Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(
                self,
                mut seq: A,
            ) -> Result<Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(v) = seq.next_element()? {
                    values.push(v);
                }
                Ok(Value::Sequence(values))
            }

            fn visit_map<A>(
                self,
                mut map: A,
            ) -> Result<Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut mapping = Mapping::new();
                while let Some((k, v)) =
                    map.next_entry()?
                {
                    mapping.insert(k, v);
                }
                Ok(Value::Mapping(mapping))
            }

            fn visit_enum<A>(
                self,
                data: A,
            ) -> Result<Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                let (tag, variant) =
                    data.variant_seed(
                        tagged::TagStringVisitor,
                    )?;
                let value: Value =
                    variant.newtype_variant()?;
                Ok(Value::Tagged(Box::new(TaggedValue {
                    tag,
                    value,
                })))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

// ---- Deserializer impl for Value ----
// Allows `T::deserialize(value)` where value: Value.

impl<'de> serde::Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Number(n) => n.deserialize_any(visitor),
            Value::String(s) => visitor.visit_string(s),
            Value::Sequence(v) => {
                let de =
                    crate::de::SeqDeserializer::new(v);
                de.deserialize_any(visitor)
            }
            Value::Mapping(v) => {
                let de =
                    crate::de::MapDeserializer::new(v);
                de.deserialize_any(visitor)
            }
            Value::Tagged(t) => visitor.visit_enum(*t),
        }
    }

    fn deserialize_option<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(s) => visitor.visit_enum(
                de::value::StrDeserializer::new(&s),
            ),
            Value::Mapping(m) => {
                if m.len() == 1 {
                    let (k, v) = m.into_iter().next()
                        .ok_or_else(|| Error::msg("empty mapping for enum"))?;
                    visitor.visit_enum(EnumDeserializer {
                        variant: k,
                        value: Some(v),
                    })
                } else {
                    Err(Error::msg(
                        "expected single-key mapping for enum",
                    ))
                }
            }
            Value::Tagged(t) => visitor.visit_enum(*t),
            _ => Err(Error::msg(
                "expected string or mapping for enum",
            )),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf unit unit_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
}

// Helper for enum deserialization from mapping
struct EnumDeserializer {
    variant: Value,
    value: Option<Value>,
}

impl<'de> de::EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(self.variant)?;
        Ok((
            variant,
            VariantDeserializer { value: self.value },
        ))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> de::VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(Value::Null) | None => Ok(()),
            Some(other) => {
                Deserialize::deserialize(other)
            }
        }
    }

    fn newtype_variant_seed<T>(
        self,
        seed: T,
    ) -> Result<T::Value, Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(v) => seed.deserialize(v),
            None => Err(Error::msg(
                "expected newtype variant value",
            )),
        }
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Sequence(v)) => {
                let de =
                    crate::de::SeqDeserializer::new(v);
                serde::Deserializer::deserialize_any(
                    de, visitor,
                )
            }
            _ => Err(Error::msg(
                "expected sequence for tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Mapping(m)) => {
                let de =
                    crate::de::MapDeserializer::new(m);
                serde::Deserializer::deserialize_any(
                    de, visitor,
                )
            }
            _ => Err(Error::msg(
                "expected mapping for struct variant",
            )),
        }
    }
}

// ---- Value Serializer (for to_value) ----

pub(crate) struct ValueSerializer;

impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeSeq;
    type SerializeTupleStruct = SerializeSeq;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(
        self,
        v: bool,
    ) -> Result<Value, Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_i16(
        self,
        v: i16,
    ) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_i32(
        self,
        v: i32,
    ) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_i64(
        self,
        v: i64,
    ) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_u8(self, v: u8) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_u16(
        self,
        v: u16,
    ) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_u32(
        self,
        v: u32,
    ) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_u64(
        self,
        v: u64,
    ) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_f32(
        self,
        v: f32,
    ) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }
    fn serialize_f64(
        self,
        v: f64,
    ) -> Result<Value, Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_char(
        self,
        v: char,
    ) -> Result<Value, Error> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(
        self,
        v: &str,
    ) -> Result<Value, Error> {
        Ok(Value::String(v.to_owned()))
    }

    fn serialize_bytes(
        self,
        _v: &[u8],
    ) -> Result<Value, Error> {
        Err(Error::msg("bytes not supported in YAML"))
    }

    fn serialize_none(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T>(
        self,
        value: &T,
    ) -> Result<Value, Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
    ) -> Result<Value, Error> {
        Ok(Value::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, Error>
    where
        T: ?Sized + Serialize,
    {
        let mut m = Mapping::new();
        m.insert(
            Value::String(variant.to_owned()),
            value.serialize(ValueSerializer)?,
        );
        Ok(Value::Mapping(m))
    }

    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> Result<SerializeSeq, Error> {
        Ok(SerializeSeq {
            vec: Vec::with_capacity(
                len.unwrap_or_default(),
            ),
        })
    }

    fn serialize_tuple(
        self,
        len: usize,
    ) -> Result<SerializeSeq, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SerializeSeq, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeTupleVariant, Error> {
        Ok(SerializeTupleVariant {
            name: variant.to_owned(),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<SerializeMap, Error> {
        Ok(SerializeMap {
            map: Mapping::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SerializeMap, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SerializeStructVariant, Error> {
        Ok(SerializeStructVariant {
            name: variant.to_owned(),
            map: Mapping::new(),
        })
    }
}

pub(crate) struct SerializeSeq {
    vec: Vec<Value>,
}

impl serde::ser::SerializeSeq for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(
        &mut self,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Sequence(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(
        &mut self,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(
            self, value,
        )
    }

    fn end(self) -> Result<Value, Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(
            self, value,
        )
    }

    fn end(self) -> Result<Value, Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

pub(crate) struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value>,
}

impl serde::ser::SerializeTupleVariant
    for SerializeTupleVariant
{
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        let mut m = Mapping::new();
        m.insert(
            Value::String(self.name),
            Value::Sequence(self.vec),
        );
        Ok(Value::Mapping(m))
    }
}

pub(crate) struct SerializeMap {
    map: Mapping,
    next_key: Option<Value>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(
        &mut self,
        key: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.next_key =
            Some(key.serialize(ValueSerializer)?);
        Ok(())
    }

    fn serialize_value<T>(
        &mut self,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| Error::msg("value before key"))?;
        self.map
            .insert(key, value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Mapping(self.map))
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(
            self, key, value,
        )
    }

    fn end(self) -> Result<Value, Error> {
        serde::ser::SerializeMap::end(self)
    }
}

pub(crate) struct SerializeStructVariant {
    name: String,
    map: Mapping,
}

impl serde::ser::SerializeStructVariant
    for SerializeStructVariant
{
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(
            Value::String(key.to_owned()),
            value.serialize(ValueSerializer)?,
        );
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        let mut m = Mapping::new();
        m.insert(
            Value::String(self.name),
            Value::Mapping(self.map),
        );
        Ok(Value::Mapping(m))
    }
}
