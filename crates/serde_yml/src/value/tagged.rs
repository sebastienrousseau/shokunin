// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::error::Error;
use crate::value::Value;
use serde::{
    de::{
        value::StrDeserializer, DeserializeSeed,
        Deserializer, EnumAccess, VariantAccess,
        Visitor,
    },
    forward_to_deserialize_any,
    ser::{Serialize, SerializeMap, Serializer},
    Deserialize,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
};

/// A YAML `!Tag`.
#[derive(Clone)]
pub struct Tag {
    /// The string representation of the tag.
    pub string: String,
}

/// A `Tag` + `Value` representing a tagged YAML value.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct TaggedValue {
    /// The tag.
    pub tag: Tag,
    /// The value.
    pub value: Value,
}

impl TaggedValue {
    /// Creates a copy of this tagged value.
    pub fn copy(&self) -> TaggedValue {
        TaggedValue {
            tag: self.tag.clone(),
            value: self.value.clone(),
        }
    }
}

impl Tag {
    /// Creates a new `Tag`.
    pub fn new(string: impl Into<String>) -> Self {
        let tag: String = string.into();
        assert!(!tag.is_empty(), "empty YAML tag not allowed");
        Tag { string: tag }
    }
}

/// Returns the portion after the leading `!`, if any.
pub fn nobang(maybe_banged: &str) -> &str {
    match maybe_banged.strip_prefix('!') {
        Some("") | None => maybe_banged,
        Some(unbanged) => unbanged,
    }
}

impl Eq for Tag {}

impl PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        nobang(&self.string) == nobang(&other.string)
    }
}

impl<T> PartialEq<T> for Tag
where
    T: ?Sized + AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        nobang(&self.string) == nobang(other.as_ref())
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        nobang(&self.string).cmp(nobang(&other.string))
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        nobang(&self.string).hash(hasher);
    }
}

impl Display for Tag {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "!{}", nobang(&self.string))
    }
}

impl Debug for Tag {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Serialize for TaggedValue {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct SerializeTag<'a>(&'a Tag);

        impl Serialize for SerializeTag<'_> {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_str(self.0)
            }
        }

        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(
            &SerializeTag(&self.tag),
            &self.value,
        )?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for TaggedValue {
    fn deserialize<D>(
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TaggedValueVisitor;

        impl<'de> Visitor<'de> for TaggedValueVisitor {
            type Value = TaggedValue;

            fn expecting(
                &self,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                f.write_str("a YAML value with a !Tag")
            }

            fn visit_enum<A>(
                self,
                data: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, contents) =
                    data.variant_seed(TagStringVisitor)?;
                let value = contents.newtype_variant()?;
                Ok(TaggedValue { tag, value })
            }
        }

        deserializer.deserialize_any(TaggedValueVisitor)
    }
}

impl<'de> Deserializer<'de> for TaggedValue {
    type Error = Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_ignored_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char
        str string bytes byte_buf option unit unit_struct
        newtype_struct seq tuple tuple_struct map struct
        enum identifier
    }
}

impl<'de> EnumAccess<'de> for TaggedValue {
    type Error = Error;
    type Variant = Value;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let tag = StrDeserializer::<Error>::new(nobang(
            &self.tag.string,
        ));
        let value = seed.deserialize(tag)?;
        Ok((value, self.value))
    }
}

impl<'de> VariantAccess<'de> for Value {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(
        self,
        seed: T,
    ) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Sequence(v) = self {
            let de = crate::de::SeqDeserializer::new(v);
            serde::Deserializer::deserialize_any(de, visitor)
        } else {
            Err(serde::de::Error::invalid_type(
                self.unexpected(),
                &"tuple variant",
            ))
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
        if let Value::Mapping(v) = self {
            let de = crate::de::MapDeserializer::new(v);
            serde::Deserializer::deserialize_any(de, visitor)
        } else {
            Err(serde::de::Error::invalid_type(
                self.unexpected(),
                &"struct variant",
            ))
        }
    }
}

pub(crate) struct TagStringVisitor;

impl Visitor<'_> for TagStringVisitor {
    type Value = Tag;

    fn expecting(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.write_str("a YAML tag string")
    }

    fn visit_str<E>(
        self,
        string: &str,
    ) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(string.to_owned())
    }

    fn visit_string<E>(
        self,
        string: String,
    ) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if string.is_empty() {
            return Err(E::custom(
                "empty YAML tag is not allowed",
            ));
        }
        Ok(Tag::new(string))
    }
}

impl<'de> DeserializeSeed<'de> for TagStringVisitor {
    type Value = Tag;

    fn deserialize<D>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(self)
    }
}

/// A tagged value with an optional tag.
#[derive(Debug)]
pub enum MaybeTag<T> {
    /// The tag.
    Tag(String),
    /// The value.
    NotTag(T),
}

/// Check if a value is a YAML tag.
pub fn check_for_tag<T>(value: &T) -> MaybeTag<String>
where
    T: ?Sized + Display,
{
    let s = format!("{}", value);
    match s.as_str() {
        "" => MaybeTag::NotTag(String::new()),
        "!" => MaybeTag::NotTag("!".to_owned()),
        tag if tag.starts_with('!') => {
            MaybeTag::Tag(tag.to_owned())
        }
        _ => MaybeTag::NotTag(s),
    }
}
