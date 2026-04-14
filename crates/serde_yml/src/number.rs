// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::error::Error;
use serde::{
    de::{Unexpected, Visitor},
    forward_to_deserialize_any, Deserialize, Deserializer,
    Serialize, Serializer,
};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    hash::{Hash, Hasher},
};

/// Represents a YAML number, whether integer or floating point.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Number {
    n: N,
}

#[derive(Copy, Clone, Debug)]
enum N {
    PositiveInteger(u64),
    NegativeInteger(i64),
    Float(f64),
}

impl Number {
    #[inline]
    pub fn is_i64(&self) -> bool {
        match self.n {
            N::PositiveInteger(v) => v <= i64::MAX as u64,
            N::NegativeInteger(_) => true,
            N::Float(_) => false,
        }
    }

    #[inline]
    pub fn is_u64(&self) -> bool {
        matches!(self.n, N::PositiveInteger(_))
    }

    #[inline]
    pub fn is_f64(&self) -> bool {
        matches!(self.n, N::Float(_))
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self.n {
            N::PositiveInteger(n) => {
                if n <= i64::MAX as u64 {
                    Some(n as i64)
                } else {
                    None
                }
            }
            N::NegativeInteger(n) => Some(n),
            N::Float(_) => None,
        }
    }

    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        match self.n {
            N::PositiveInteger(n) => Some(n),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self.n {
            N::PositiveInteger(n) => Some(n as f64),
            N::NegativeInteger(n) => Some(n as f64),
            N::Float(n) => Some(n),
        }
    }

    #[inline]
    pub fn is_nan(&self) -> bool {
        matches!(self.n, N::Float(f) if f.is_nan())
    }

    #[inline]
    pub fn is_infinite(&self) -> bool {
        matches!(self.n, N::Float(f) if f.is_infinite())
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        match self.n {
            N::PositiveInteger(_) | N::NegativeInteger(_) => true,
            N::Float(f) => f.is_finite(),
        }
    }

    pub(crate) fn total_cmp(&self, other: &Self) -> Ordering {
        self.n.total_cmp(&other.n)
    }
}

impl Display for Number {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self.n {
            N::PositiveInteger(i) => write!(formatter, "{}", i),
            N::NegativeInteger(i) => write!(formatter, "{}", i),
            N::Float(f) if f.is_nan() => {
                formatter.write_str(".nan")
            }
            N::Float(f) if f.is_infinite() => {
                if f.is_sign_negative() {
                    formatter.write_str("-.inf")
                } else {
                    formatter.write_str(".inf")
                }
            }
            N::Float(f) => {
                if f.fract() == 0.0 && f.is_finite() {
                    write!(formatter, "{:.1}", f)
                } else {
                    write!(formatter, "{}", f)
                }
            }
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Number({})", self)
    }
}

impl PartialEq for N {
    fn eq(&self, other: &N) -> bool {
        match (*self, *other) {
            (N::PositiveInteger(a), N::PositiveInteger(b)) => {
                a == b
            }
            (N::NegativeInteger(a), N::NegativeInteger(b)) => {
                a == b
            }
            (N::Float(a), N::Float(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            _ => false,
        }
    }
}

impl PartialOrd for N {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        match (*self, *other) {
            (N::Float(a), N::Float(b)) => {
                if a.is_nan() && b.is_nan() {
                    Some(Ordering::Equal)
                } else {
                    a.partial_cmp(&b)
                }
            }
            _ => Some(self.total_cmp(other)),
        }
    }
}

impl N {
    fn total_cmp(&self, other: &Self) -> Ordering {
        match (*self, *other) {
            (N::PositiveInteger(a), N::PositiveInteger(b)) => {
                a.cmp(&b)
            }
            (N::NegativeInteger(a), N::NegativeInteger(b)) => {
                a.cmp(&b)
            }
            (N::NegativeInteger(_), N::PositiveInteger(_)) => {
                Ordering::Less
            }
            (N::PositiveInteger(_), N::NegativeInteger(_)) => {
                Ordering::Greater
            }
            (N::Float(a), N::Float(b)) => a
                .partial_cmp(&b)
                .unwrap_or(Ordering::Equal),
            (_, N::Float(_)) => Ordering::Less,
            (N::Float(_), _) => Ordering::Greater,
        }
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.n {
            N::PositiveInteger(u) => u.hash(state),
            N::NegativeInteger(i) => i.hash(state),
            N::Float(f) => f.to_bits().hash(state),
        }
    }
}

impl Eq for Number {}

impl Serialize for Number {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.n {
            N::PositiveInteger(i) => serializer.serialize_u64(i),
            N::NegativeInteger(i) => serializer.serialize_i64(i),
            N::Float(f) => serializer.serialize_f64(f),
        }
    }
}

struct NumberVisitor;

impl Visitor<'_> for NumberVisitor {
    type Value = Number;

    fn expecting(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.write_str("a number")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Number, E> {
        Ok(v.into())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Number, E> {
        Ok(v.into())
    }

    fn visit_f64<E>(self, v: f64) -> Result<Number, E> {
        Ok(v.into())
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(
        deserializer: D,
    ) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NumberVisitor)
    }
}

impl<'de> Deserializer<'de> for Number {
    type Error = Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.n {
            N::PositiveInteger(i) => visitor.visit_u64(i),
            N::NegativeInteger(i) => visitor.visit_i64(i),
            N::Float(f) => visitor.visit_f64(f),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct map
        struct enum identifier ignored_any
    }
}

impl<'de> Deserializer<'de> for &Number {
    type Error = Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.n {
            N::PositiveInteger(i) => visitor.visit_u64(i),
            N::NegativeInteger(i) => visitor.visit_i64(i),
            N::Float(f) => visitor.visit_f64(f),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct map
        struct enum identifier ignored_any
    }
}

pub(crate) fn unexpected(
    number: &Number,
) -> Unexpected<'_> {
    match number.n {
        N::PositiveInteger(u) => Unexpected::Unsigned(u),
        N::NegativeInteger(i) => Unexpected::Signed(i),
        N::Float(f) => Unexpected::Float(f),
    }
}

macro_rules! from_signed {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(i: $ty) -> Self {
                    if i < 0 {
                        Number {
                            n: N::NegativeInteger(i64::from(i)),
                        }
                    } else {
                        Number {
                            n: N::PositiveInteger(i as u64),
                        }
                    }
                }
            }
        )*
    };
}

macro_rules! from_unsigned {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(u: $ty) -> Self {
                    Number {
                        n: N::PositiveInteger(u64::from(u)),
                    }
                }
            }
        )*
    };
}

from_signed!(i8 i16 i32);
from_unsigned!(u8 u16 u32);

impl From<i64> for Number {
    #[inline]
    fn from(i: i64) -> Self {
        if i < 0 {
            Number {
                n: N::NegativeInteger(i),
            }
        } else {
            Number {
                n: N::PositiveInteger(i as u64),
            }
        }
    }
}

impl From<u64> for Number {
    #[inline]
    fn from(u: u64) -> Self {
        Number {
            n: N::PositiveInteger(u),
        }
    }
}

impl From<isize> for Number {
    #[inline]
    fn from(i: isize) -> Self {
        Number::from(i as i64)
    }
}

impl From<usize> for Number {
    #[inline]
    fn from(u: usize) -> Self {
        Number::from(u as u64)
    }
}

impl From<f32> for Number {
    fn from(f: f32) -> Self {
        Number::from(f as f64)
    }
}

impl From<f64> for Number {
    fn from(mut f: f64) -> Self {
        if f.is_nan() {
            f = f64::NAN.copysign(1.0);
        }
        Number { n: N::Float(f) }
    }
}
