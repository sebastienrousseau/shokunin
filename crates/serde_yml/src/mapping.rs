// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::Value;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    cmp::Ordering,
    collections::hash_map::DefaultHasher,
    fmt::{self, Display},
    hash::{Hash, Hasher},
    mem,
};

/// A YAML mapping in which the keys and values are both
/// `serde_yml::Value`.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Mapping {
    /// The underlying map.
    pub map: IndexMap<Value, Value>,
}

impl fmt::Debug for Mapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl Mapping {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Mapping {
            map: IndexMap::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }

    #[inline]
    pub fn insert(
        &mut self,
        k: Value,
        v: Value,
    ) -> Option<Value> {
        self.map.insert(k, v)
    }

    #[inline]
    pub fn contains_key<I: Index>(&self, index: I) -> bool {
        index.is_key_into(self)
    }

    #[inline]
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.index_into(self)
    }

    #[inline]
    pub fn get_mut<I: Index>(
        &mut self,
        index: I,
    ) -> Option<&mut Value> {
        index.index_into_mut(self)
    }

    #[inline]
    pub fn entry(&mut self, k: Value) -> Entry<'_> {
        match self.map.entry(k) {
            indexmap::map::Entry::Occupied(occupied) => {
                Entry::Occupied(OccupiedEntry { occupied })
            }
            indexmap::map::Entry::Vacant(vacant) => {
                Entry::Vacant(VacantEntry { vacant })
            }
        }
    }

    #[inline]
    pub fn remove<I: Index>(
        &mut self,
        index: I,
    ) -> Option<Value> {
        self.swap_remove(index)
    }

    #[inline]
    pub fn remove_entry<I: Index>(
        &mut self,
        index: I,
    ) -> Option<(Value, Value)> {
        self.swap_remove_entry(index)
    }

    #[inline]
    pub fn swap_remove<I: Index>(
        &mut self,
        index: I,
    ) -> Option<Value> {
        index.swap_remove_from(self)
    }

    #[inline]
    pub fn swap_remove_entry<I: Index>(
        &mut self,
        index: I,
    ) -> Option<(Value, Value)> {
        index.swap_remove_entry_from(self)
    }

    #[inline]
    pub fn shift_remove<I: Index>(
        &mut self,
        index: I,
    ) -> Option<Value> {
        index.shift_remove_from(self)
    }

    #[inline]
    pub fn shift_remove_entry<I: Index>(
        &mut self,
        index: I,
    ) -> Option<(Value, Value)> {
        index.shift_remove_entry_from(self)
    }

    #[inline]
    pub fn retain<F>(&mut self, keep: F)
    where
        F: FnMut(&Value, &mut Value) -> bool,
    {
        self.map.retain(keep);
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            iter: self.map.iter(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }

    pub fn keys(&self) -> Keys<'_> {
        Keys {
            iter: self.map.keys(),
        }
    }

    pub fn into_keys(self) -> IntoKeys {
        IntoKeys {
            iter: self.map.into_keys(),
        }
    }

    pub fn values(&self) -> Values<'_> {
        Values {
            iter: self.map.values(),
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_> {
        ValuesMut {
            iter: self.map.values_mut(),
        }
    }

    pub fn into_values(self) -> IntoValues {
        IntoValues {
            iter: self.map.into_values(),
        }
    }
}

// ---- Index trait ----

/// Sealed trait to prevent external implementations.
mod private {
    pub trait Sealed {}
}

/// Types that can index into a `Mapping`.
pub trait Index: private::Sealed {
    #[doc(hidden)]
    fn is_key_into(&self, v: &Mapping) -> bool;
    #[doc(hidden)]
    fn index_into<'a>(
        &self,
        v: &'a Mapping,
    ) -> Option<&'a Value>;
    #[doc(hidden)]
    fn index_into_mut<'a>(
        &self,
        v: &'a mut Mapping,
    ) -> Option<&'a mut Value>;
    #[doc(hidden)]
    fn swap_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value>;
    #[doc(hidden)]
    fn swap_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)>;
    #[doc(hidden)]
    fn shift_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value>;
    #[doc(hidden)]
    fn shift_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)>;
}

struct HashLikeValue<'a>(&'a str);

impl indexmap::Equivalent<Value> for HashLikeValue<'_> {
    fn equivalent(&self, key: &Value) -> bool {
        match key {
            Value::String(s) => self.0 == s,
            _ => false,
        }
    }
}

impl Hash for HashLikeValue<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        const STRING: Value = Value::String(String::new());
        mem::discriminant(&STRING).hash(state);
        self.0.hash(state);
    }
}

impl private::Sealed for Value {}
impl Index for Value {
    fn is_key_into(&self, v: &Mapping) -> bool {
        v.map.contains_key(self)
    }
    fn index_into<'a>(
        &self,
        v: &'a Mapping,
    ) -> Option<&'a Value> {
        v.map.get(self)
    }
    fn index_into_mut<'a>(
        &self,
        v: &'a mut Mapping,
    ) -> Option<&'a mut Value> {
        v.map.get_mut(self)
    }
    fn swap_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value> {
        v.map.swap_remove(self)
    }
    fn swap_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)> {
        v.map.swap_remove_entry(self)
    }
    fn shift_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value> {
        v.map.shift_remove(self)
    }
    fn shift_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)> {
        v.map.shift_remove_entry(self)
    }
}

impl private::Sealed for str {}
impl Index for str {
    fn is_key_into(&self, v: &Mapping) -> bool {
        v.map.contains_key(&HashLikeValue(self))
    }
    fn index_into<'a>(
        &self,
        v: &'a Mapping,
    ) -> Option<&'a Value> {
        v.map.get(&HashLikeValue(self))
    }
    fn index_into_mut<'a>(
        &self,
        v: &'a mut Mapping,
    ) -> Option<&'a mut Value> {
        v.map.get_mut(&HashLikeValue(self))
    }
    fn swap_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value> {
        v.map.swap_remove(&HashLikeValue(self))
    }
    fn swap_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)> {
        v.map.swap_remove_entry(&HashLikeValue(self))
    }
    fn shift_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value> {
        v.map.shift_remove(&HashLikeValue(self))
    }
    fn shift_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)> {
        v.map.shift_remove_entry(&HashLikeValue(self))
    }
}

impl private::Sealed for String {}
impl Index for String {
    fn is_key_into(&self, v: &Mapping) -> bool {
        self.as_str().is_key_into(v)
    }
    fn index_into<'a>(
        &self,
        v: &'a Mapping,
    ) -> Option<&'a Value> {
        self.as_str().index_into(v)
    }
    fn index_into_mut<'a>(
        &self,
        v: &'a mut Mapping,
    ) -> Option<&'a mut Value> {
        self.as_str().index_into_mut(v)
    }
    fn swap_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value> {
        self.as_str().swap_remove_from(v)
    }
    fn swap_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)> {
        self.as_str().swap_remove_entry_from(v)
    }
    fn shift_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value> {
        self.as_str().shift_remove_from(v)
    }
    fn shift_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)> {
        self.as_str().shift_remove_entry_from(v)
    }
}

impl<T> private::Sealed for &T where T: ?Sized + private::Sealed {}
impl<T> Index for &T
where
    T: ?Sized + Index,
{
    fn is_key_into(&self, v: &Mapping) -> bool {
        (**self).is_key_into(v)
    }
    fn index_into<'a>(
        &self,
        v: &'a Mapping,
    ) -> Option<&'a Value> {
        (**self).index_into(v)
    }
    fn index_into_mut<'a>(
        &self,
        v: &'a mut Mapping,
    ) -> Option<&'a mut Value> {
        (**self).index_into_mut(v)
    }
    fn swap_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value> {
        (**self).swap_remove_from(v)
    }
    fn swap_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)> {
        (**self).swap_remove_entry_from(v)
    }
    fn shift_remove_from(
        &self,
        v: &mut Mapping,
    ) -> Option<Value> {
        (**self).shift_remove_from(v)
    }
    fn shift_remove_entry_from(
        &self,
        v: &mut Mapping,
    ) -> Option<(Value, Value)> {
        (**self).shift_remove_entry_from(v)
    }
}

// ---- Hash, PartialOrd ----

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Mapping {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut xor = 0u64;
        for (k, v) in self {
            let mut hasher = DefaultHasher::new();
            k.hash(&mut hasher);
            v.hash(&mut hasher);
            xor ^= hasher.finish();
        }
        xor.hash(state);
    }
}

impl PartialOrd for Mapping {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        let mut self_entries = Vec::from_iter(self);
        let mut other_entries = Vec::from_iter(other);
        let total_cmp_fn =
            |&(a, _): &_, &(b, _): &_| crate::value::total_cmp(a, b);
        self_entries.sort_by(total_cmp_fn);
        other_entries.sort_by(total_cmp_fn);
        self_entries.partial_cmp(&other_entries)
    }
}

// ---- std::ops::Index ----

impl<I> std::ops::Index<I> for Mapping
where
    I: Index,
{
    type Output = Value;

    #[inline]
    #[track_caller]
    fn index(&self, index: I) -> &Value {
        // This panic is intentional (matches original API)
        #[allow(clippy::expect_used)]
        index.index_into(self).expect("key not found")
    }
}

impl<I> std::ops::IndexMut<I> for Mapping
where
    I: Index,
{
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: I) -> &mut Value {
        #[allow(clippy::expect_used)]
        index.index_into_mut(self).expect("key not found")
    }
}

impl Extend<(Value, Value)> for Mapping {
    fn extend<I: IntoIterator<Item = (Value, Value)>>(
        &mut self,
        iter: I,
    ) {
        self.map.extend(iter);
    }
}

impl FromIterator<(Value, Value)> for Mapping {
    fn from_iter<I: IntoIterator<Item = (Value, Value)>>(
        iter: I,
    ) -> Self {
        Mapping {
            map: IndexMap::from_iter(iter),
        }
    }
}

// ---- Iterator types ----

macro_rules! delegate_iterator {
    (($name:ident $($g:tt)*) => $item:ty) => {
        impl $($g)* Iterator for $name $($g)* {
            type Item = $item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }
        impl $($g)* ExactSizeIterator for $name $($g)* {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }
    };
}

#[derive(Debug)]
pub struct Iter<'a> {
    iter: indexmap::map::Iter<'a, Value, Value>,
}
delegate_iterator!((Iter<'a>) => (&'a Value, &'a Value));

impl<'a> IntoIterator for &'a Mapping {
    type Item = (&'a Value, &'a Value);
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.map.iter(),
        }
    }
}

#[derive(Debug)]
pub struct IterMut<'a> {
    iter: indexmap::map::IterMut<'a, Value, Value>,
}
delegate_iterator!((IterMut<'a>) => (&'a Value, &'a mut Value));

impl<'a> IntoIterator for &'a mut Mapping {
    type Item = (&'a Value, &'a mut Value);
    type IntoIter = IterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }
}

#[derive(Debug)]
pub struct IntoIter {
    iter: indexmap::map::IntoIter<Value, Value>,
}
delegate_iterator!((IntoIter) => (Value, Value));

impl IntoIterator for Mapping {
    type Item = (Value, Value);
    type IntoIter = IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

#[derive(Debug)]
pub struct Keys<'a> {
    iter: indexmap::map::Keys<'a, Value, Value>,
}
delegate_iterator!((Keys<'a>) => &'a Value);

#[derive(Debug)]
pub struct IntoKeys {
    iter: indexmap::map::IntoKeys<Value, Value>,
}
delegate_iterator!((IntoKeys) => Value);

#[derive(Debug)]
pub struct Values<'a> {
    iter: indexmap::map::Values<'a, Value, Value>,
}
delegate_iterator!((Values<'a>) => &'a Value);

#[derive(Debug)]
pub struct ValuesMut<'a> {
    iter: indexmap::map::ValuesMut<'a, Value, Value>,
}
delegate_iterator!((ValuesMut<'a>) => &'a mut Value);

#[derive(Debug)]
pub struct IntoValues {
    iter: indexmap::map::IntoValues<Value, Value>,
}
delegate_iterator!((IntoValues) => Value);

// ---- Entry types ----

#[derive(Debug)]
pub enum Entry<'a> {
    Occupied(OccupiedEntry<'a>),
    Vacant(VacantEntry<'a>),
}

#[derive(Debug)]
pub struct OccupiedEntry<'a> {
    occupied:
        indexmap::map::OccupiedEntry<'a, Value, Value>,
}

#[derive(Debug)]
pub struct VacantEntry<'a> {
    vacant: indexmap::map::VacantEntry<'a, Value, Value>,
}

impl<'a> Entry<'a> {
    pub fn key(&self) -> &Value {
        match self {
            Entry::Vacant(e) => e.key(),
            Entry::Occupied(e) => e.key(),
        }
    }

    pub fn or_insert(self, default: Value) -> &'a mut Value {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    pub fn or_insert_with<F>(
        self,
        default: F,
    ) -> &'a mut Value
    where
        F: FnOnce() -> Value,
    {
        match self {
            Entry::Vacant(entry) => entry.insert(default()),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }
}

impl<'a> OccupiedEntry<'a> {
    #[inline]
    pub fn key(&self) -> &Value {
        self.occupied.key()
    }
    #[inline]
    pub fn get(&self) -> &Value {
        self.occupied.get()
    }
    #[inline]
    pub fn get_mut(&mut self) -> &mut Value {
        self.occupied.get_mut()
    }
    #[inline]
    pub fn into_mut(self) -> &'a mut Value {
        self.occupied.into_mut()
    }
    #[inline]
    pub fn insert(&mut self, value: Value) -> Value {
        self.occupied.insert(value)
    }
    #[inline]
    pub fn remove(self) -> Value {
        self.occupied.swap_remove()
    }
    #[inline]
    pub fn remove_entry(self) -> (Value, Value) {
        self.occupied.swap_remove_entry()
    }
}

impl<'a> VacantEntry<'a> {
    #[inline]
    pub fn key(&self) -> &Value {
        self.vacant.key()
    }
    #[inline]
    pub fn into_key(self) -> Value {
        self.vacant.into_key()
    }
    #[inline]
    pub fn insert(self, value: Value) -> &'a mut Value {
        self.vacant.insert(value)
    }
}

// ---- Serialize / Deserialize ----

impl Serialize for Mapping {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map =
            serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Mapping {
    fn deserialize<D>(
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MappingVisitor;

        impl<'de> serde::de::Visitor<'de> for MappingVisitor {
            type Value = Mapping;

            fn expecting(
                &self,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                f.write_str("a YAML mapping")
            }

            fn visit_unit<E>(
                self,
            ) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Mapping::new())
            }

            fn visit_map<A>(
                self,
                mut data: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut mapping = Mapping::new();
                while let Some(key) = data.next_key()? {
                    match mapping.entry(key) {
                        Entry::Occupied(entry) => {
                            return Err(
                                serde::de::Error::custom(
                                    format!(
                                        "duplicate key: {:?}",
                                        entry.key()
                                    ),
                                ),
                            );
                        }
                        Entry::Vacant(entry) => {
                            let value =
                                data.next_value()?;
                            entry.insert(value);
                        }
                    }
                }
                Ok(mapping)
            }
        }

        deserializer.deserialize_map(MappingVisitor)
    }
}

/// Display for Mapping.
impl Display for Mapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, (k, v)) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}: {:?}", k, v)?;
        }
        write!(f, "}}")
    }
}
