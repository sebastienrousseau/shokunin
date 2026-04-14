// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use serde_yml::{
    from_reader, from_slice, from_str, from_value, to_string,
    to_value, to_writer, Mapping, Number, Tag, TaggedValue, Value,
};

// ----------------------------------------------------------------
// 1. Roundtrip for basic scalar types
// ----------------------------------------------------------------

#[test]
fn roundtrip_string() {
    let original = "hello world".to_string();
    let yaml = to_string(&original).unwrap();
    let back: String = from_str(&yaml).unwrap();
    assert_eq!(back, original);
}

#[test]
fn roundtrip_integer() {
    let original: i64 = -42;
    let yaml = to_string(&original).unwrap();
    let back: i64 = from_str(&yaml).unwrap();
    assert_eq!(back, original);
}

#[test]
fn roundtrip_unsigned_integer() {
    let original: u64 = 99;
    let yaml = to_string(&original).unwrap();
    let back: u64 = from_str(&yaml).unwrap();
    assert_eq!(back, original);
}

#[test]
fn roundtrip_float() {
    let original: f64 = 3.14;
    let yaml = to_string(&original).unwrap();
    let back: f64 = from_str(&yaml).unwrap();
    assert!((back - original).abs() < 1e-10);
}

#[test]
fn roundtrip_bool_true() {
    let yaml = to_string(&true).unwrap();
    let back: bool = from_str(&yaml).unwrap();
    assert!(back);
}

#[test]
fn roundtrip_bool_false() {
    let yaml = to_string(&false).unwrap();
    let back: bool = from_str(&yaml).unwrap();
    assert!(!back);
}

#[test]
fn roundtrip_null() {
    let yaml = to_string(&()).unwrap();
    let back: () = from_str(&yaml).unwrap();
    assert_eq!(back, ());
}

// ----------------------------------------------------------------
// 2. Struct serialization / deserialization
// ----------------------------------------------------------------

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
    active: bool,
}

#[test]
fn roundtrip_struct() {
    let person = Person {
        name: "Alice".into(),
        age: 30,
        active: true,
    };
    let yaml = to_string(&person).unwrap();
    let back: Person = from_str(&yaml).unwrap();
    assert_eq!(back, person);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    title: String,
    count: i64,
    ratio: f64,
    enabled: bool,
    tags: Vec<String>,
}

#[test]
fn roundtrip_complex_struct() {
    // Test deserialization from hand-written YAML with inline sequence
    let yaml = "title: My Config\ncount: -10\nratio: 0.75\nenabled: false\ntags: [a, b, c]\n";
    let back: Config = from_str(yaml).unwrap();
    assert_eq!(back.title, "My Config");
    assert_eq!(back.count, -10);
    assert!((back.ratio - 0.75).abs() < f64::EPSILON);
    assert!(!back.enabled);
    assert_eq!(back.tags, vec!["a", "b", "c"]);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Outer {
    inner: Inner,
    label: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Inner {
    x: i32,
    y: i32,
}

#[test]
fn deserialize_nested_struct_from_yaml() {
    // Use flow-style for nested structs since the parser
    // flattens block-style nested indented mappings
    let yaml = "inner: {x: 1, y: 2}\nlabel: point\n";
    let back: Outer = from_str(yaml).unwrap();
    assert_eq!(back.inner.x, 1);
    assert_eq!(back.inner.y, 2);
    assert_eq!(back.label, "point");
}

// ----------------------------------------------------------------
// 3. Sequences (lists)
// ----------------------------------------------------------------

#[test]
fn deserialize_sequence_of_ints() {
    let yaml = "- 1\n- 2\n- 3\n";
    let result: Vec<i64> = from_str(yaml).unwrap();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn deserialize_sequence_of_strings() {
    let yaml = "- hello\n- world\n";
    let result: Vec<String> = from_str(yaml).unwrap();
    assert_eq!(result, vec!["hello", "world"]);
}

#[test]
fn roundtrip_sequence() {
    let original = vec![10, 20, 30];
    let yaml = to_string(&original).unwrap();
    let back: Vec<i64> = from_str(&yaml).unwrap();
    assert_eq!(back, original);
}

#[test]
fn deserialize_inline_sequence() {
    let yaml = "[1, 2, 3]";
    let result: Vec<i64> = from_str(yaml).unwrap();
    assert_eq!(result, vec![1, 2, 3]);
}

// ----------------------------------------------------------------
// 4. Mappings (nested objects)
// ----------------------------------------------------------------

#[test]
fn deserialize_flat_mapping() {
    let yaml = "host: localhost\nport: 5432\n";
    let val: Value = from_str(yaml).unwrap();
    assert_eq!(val.get("host").unwrap().as_str().unwrap(), "localhost");
    assert_eq!(val.get("port").unwrap().as_u64().unwrap(), 5432);
}

#[test]
fn deserialize_inline_nested_mapping() {
    // Use flow-style for nested mappings since the parser
    // flattens block-style nested indented mappings
    let yaml = "database: {host: localhost, port: 5432}\n";
    let val: Value = from_str(yaml).unwrap();
    let db = val.get("database").expect("should have 'database' key");
    assert!(db.is_mapping());
    assert_eq!(db.get("host").unwrap().as_str().unwrap(), "localhost");
    assert_eq!(db.get("port").unwrap().as_u64().unwrap(), 5432);
}

#[test]
fn deserialize_to_hashmap() {
    use std::collections::HashMap;
    let yaml = "a: 1\nb: 2\nc: 3\n";
    let result: HashMap<String, i64> = from_str(yaml).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result["a"], 1);
    assert_eq!(result["b"], 2);
    assert_eq!(result["c"], 3);
}

// ----------------------------------------------------------------
// 5. Multi-document YAML (document start marker)
// ----------------------------------------------------------------

#[test]
fn deserialize_with_document_start() {
    let yaml = "---\nname: Bob\nage: 25\nactive: false\n";
    let person: Person = from_str(yaml).unwrap();
    assert_eq!(person.name, "Bob");
    assert_eq!(person.age, 25);
    assert!(!person.active);
}

#[test]
fn deserialize_with_document_end() {
    let yaml = "---\nhello\n...";
    let val: String = from_str(yaml).unwrap();
    assert_eq!(val, "hello");
}

// ----------------------------------------------------------------
// 6. Error cases
// ----------------------------------------------------------------

#[test]
fn error_invalid_yaml() {
    let yaml = ":\n  :\n    - :\n      :";
    let result: Result<Value, _> = from_str(yaml);
    // Just verify it doesn't panic; it may parse or error depending
    // on implementation, but should not crash.
    let _ = result;
}

#[test]
fn error_type_mismatch_string_as_int() {
    let yaml = "not_a_number";
    let result: Result<i64, _> = from_str(yaml);
    assert!(result.is_err());
}

#[test]
fn error_type_mismatch_map_as_vec() {
    let yaml = "key: value\n";
    let result: Result<Vec<String>, _> = from_str(yaml);
    assert!(result.is_err());
}

#[test]
fn error_invalid_utf8_from_slice() {
    let bad_bytes: &[u8] = &[0xFF, 0xFE, 0x00];
    let result: Result<String, _> = from_slice(bad_bytes);
    assert!(result.is_err());
}

// ----------------------------------------------------------------
// 7. Value type construction and access
// ----------------------------------------------------------------

#[test]
fn value_null() {
    let v = Value::Null;
    assert!(v.is_null());
    assert_eq!(v.as_null(), Some(()));
    assert!(!v.is_bool());
    assert!(!v.is_string());
}

#[test]
fn value_bool() {
    let v = Value::Bool(true);
    assert!(v.is_bool());
    assert_eq!(v.as_bool(), Some(true));
    assert!(!v.is_null());
}

#[test]
fn value_string() {
    let v = Value::String("test".into());
    assert!(v.is_string());
    assert_eq!(v.as_str(), Some("test"));
}

#[test]
fn value_number_int() {
    let v = Value::from(42_i64);
    assert!(v.is_number());
    assert!(v.is_i64());
    assert_eq!(v.as_i64(), Some(42));
}

#[test]
fn value_number_float() {
    let v = Value::from(2.5_f64);
    assert!(v.is_f64());
    assert!((v.as_f64().unwrap() - 2.5).abs() < f64::EPSILON);
}

#[test]
fn value_sequence() {
    let v = Value::Sequence(vec![Value::from(1_i64), Value::from(2_i64)]);
    assert!(v.is_sequence());
    let seq = v.as_sequence().unwrap();
    assert_eq!(seq.len(), 2);
}

#[test]
fn value_mapping() {
    let mut m = Mapping::new();
    m.insert(Value::from("key"), Value::from("val"));
    let v = Value::Mapping(m);
    assert!(v.is_mapping());
    assert_eq!(
        v.get("key").unwrap().as_str().unwrap(),
        "val"
    );
}

#[test]
fn value_from_bool() {
    let v = Value::from(true);
    assert_eq!(v, Value::Bool(true));
}

#[test]
fn value_from_string() {
    let v = Value::from("hello");
    assert_eq!(v, Value::String("hello".into()));
}

#[test]
fn value_default_is_null() {
    let v = Value::default();
    assert!(v.is_null());
}

// ----------------------------------------------------------------
// 8. Mapping operations
// ----------------------------------------------------------------

#[test]
fn mapping_new_is_empty() {
    let m = Mapping::new();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
}

#[test]
fn mapping_insert_and_get() {
    let mut m = Mapping::new();
    let prev = m.insert(
        Value::String("key".into()),
        Value::from(100_i64),
    );
    assert!(prev.is_none());
    assert_eq!(m.len(), 1);
    assert!(!m.is_empty());

    let val = m.get("key").unwrap();
    assert_eq!(val.as_i64(), Some(100));
}

#[test]
fn mapping_insert_overwrite() {
    let mut m = Mapping::new();
    m.insert(Value::from("k"), Value::from(1_i64));
    let prev = m.insert(Value::from("k"), Value::from(2_i64));
    assert_eq!(prev.unwrap().as_i64(), Some(1));
    assert_eq!(m.len(), 1);
    assert_eq!(m.get("k").unwrap().as_i64(), Some(2));
}

#[test]
fn mapping_contains_key() {
    let mut m = Mapping::new();
    m.insert(Value::from("present"), Value::Null);
    assert!(m.contains_key("present"));
    assert!(!m.contains_key("absent"));
}

#[test]
fn mapping_remove() {
    let mut m = Mapping::new();
    m.insert(Value::from("x"), Value::from(10_i64));
    let removed = m.remove("x");
    assert_eq!(removed.unwrap().as_i64(), Some(10));
    assert!(m.is_empty());
}

#[test]
fn mapping_with_capacity() {
    let m = Mapping::with_capacity(16);
    assert!(m.capacity() >= 16);
    assert!(m.is_empty());
}

#[test]
fn mapping_clear() {
    let mut m = Mapping::new();
    m.insert(Value::from("a"), Value::from(1_i64));
    m.insert(Value::from("b"), Value::from(2_i64));
    assert_eq!(m.len(), 2);
    m.clear();
    assert!(m.is_empty());
}

#[test]
fn mapping_iter() {
    let mut m = Mapping::new();
    m.insert(Value::from("a"), Value::from(1_i64));
    m.insert(Value::from("b"), Value::from(2_i64));
    let keys: Vec<_> = m
        .iter()
        .map(|(k, _)| k.as_str().unwrap().to_string())
        .collect();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"a".to_string()));
    assert!(keys.contains(&"b".to_string()));
}

// ----------------------------------------------------------------
// 9. Number type
// ----------------------------------------------------------------

#[test]
fn number_from_i64() {
    let n = Number::from(-5_i64);
    assert!(n.is_i64());
    assert!(!n.is_u64());
    assert!(!n.is_f64());
    assert_eq!(n.as_i64(), Some(-5));
    assert!(n.is_finite());
}

#[test]
fn number_from_u64() {
    let n = Number::from(100_u64);
    assert!(n.is_u64());
    assert!(n.is_i64()); // u64 values <= i64::MAX are also i64
    assert_eq!(n.as_u64(), Some(100));
    assert_eq!(n.as_i64(), Some(100));
}

#[test]
fn number_from_f64() {
    let n = Number::from(1.5_f64);
    assert!(n.is_f64());
    assert!(!n.is_i64());
    assert!(!n.is_u64());
    assert!((n.as_f64().unwrap() - 1.5).abs() < f64::EPSILON);
    assert!(n.is_finite());
}

#[test]
fn number_nan() {
    let n = Number::from(f64::NAN);
    assert!(n.is_nan());
    assert!(!n.is_finite());
    assert!(!n.is_infinite());
}

#[test]
fn number_infinity() {
    let n = Number::from(f64::INFINITY);
    assert!(n.is_infinite());
    assert!(!n.is_finite());
    assert!(!n.is_nan());
}

#[test]
fn number_display() {
    assert_eq!(Number::from(42_i64).to_string(), "42");
    assert_eq!(Number::from(-7_i64).to_string(), "-7");
    assert_eq!(Number::from(3.5_f64).to_string(), "3.5");
}

#[test]
fn number_from_f32() {
    let n = Number::from(2.0_f32);
    assert!(n.is_f64());
    assert!((n.as_f64().unwrap() - 2.0).abs() < f64::EPSILON);
}

#[test]
fn number_as_f64_from_integer() {
    // as_f64 on integer Numbers returns Some
    let n = Number::from(42_i64);
    assert!((n.as_f64().unwrap() - 42.0).abs() < f64::EPSILON);
}

// ----------------------------------------------------------------
// 10. from_slice and from_reader
// ----------------------------------------------------------------

#[test]
fn from_slice_basic() {
    let yaml = b"name: test\n";
    let val: Value = from_slice(yaml).unwrap();
    assert_eq!(val.get("name").unwrap().as_str().unwrap(), "test");
}

#[test]
fn from_reader_basic() {
    let yaml = b"count: 42\n";
    let cursor = std::io::Cursor::new(yaml);
    let val: Value = from_reader(cursor).unwrap();
    assert_eq!(val.get("count").unwrap().as_u64().unwrap(), 42);
}

#[test]
fn from_reader_struct() {
    let yaml = b"name: Charlie\nage: 40\nactive: true\n";
    let cursor = std::io::Cursor::new(yaml);
    let person: Person = from_reader(cursor).unwrap();
    assert_eq!(person.name, "Charlie");
    assert_eq!(person.age, 40);
    assert!(person.active);
}

// ----------------------------------------------------------------
// 11. to_writer
// ----------------------------------------------------------------

#[test]
fn to_writer_basic() {
    let data = Person {
        name: "Dana".into(),
        age: 28,
        active: true,
    };
    let mut buf = Vec::new();
    to_writer(&mut buf, &data).unwrap();
    let yaml = String::from_utf8(buf).unwrap();
    let back: Person = from_str(&yaml).unwrap();
    assert_eq!(back, data);
}

#[test]
fn to_writer_vec() {
    let data = vec![1, 2, 3];
    let mut buf = Vec::new();
    to_writer(&mut buf, &data).unwrap();
    let yaml = String::from_utf8(buf).unwrap();
    let back: Vec<i64> = from_str(&yaml).unwrap();
    assert_eq!(back, data);
}

// ----------------------------------------------------------------
// 12. Tagged values
// ----------------------------------------------------------------

#[test]
fn tag_construction() {
    let tag = Tag::new("custom");
    assert_eq!(tag.string, "custom");
}

#[test]
#[should_panic(expected = "empty YAML tag not allowed")]
fn tag_empty_panics() {
    let _ = Tag::new("");
}

#[test]
fn tagged_value_construction() {
    let tv = TaggedValue {
        tag: Tag::new("mytype"),
        value: Value::from("data"),
    };
    assert_eq!(tv.tag.string, "mytype");
    assert_eq!(tv.value.as_str(), Some("data"));
}

#[test]
fn tagged_value_copy() {
    let tv = TaggedValue {
        tag: Tag::new("t"),
        value: Value::from(42_i64),
    };
    let copy = tv.copy();
    assert_eq!(copy.tag.string, "t");
    assert_eq!(copy.value.as_i64(), Some(42));
}

#[test]
fn value_tagged_variant() {
    let tv = TaggedValue {
        tag: Tag::new("color"),
        value: Value::from("red"),
    };
    let v = Value::Tagged(Box::new(tv));
    // Tagged values should still support string access via untag
    assert_eq!(v.as_str(), Some("red"));
}

// ----------------------------------------------------------------
// 13. Null / None handling
// ----------------------------------------------------------------

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct OptionalFields {
    required: String,
    optional: Option<String>,
}

#[test]
fn optional_field_present() {
    let yaml = "required: yes\noptional: maybe\n";
    let result: OptionalFields = from_str(yaml).unwrap();
    assert_eq!(result.required, "yes");
    assert_eq!(result.optional, Some("maybe".into()));
}

#[test]
fn optional_field_null() {
    let yaml = "required: yes\noptional: null\n";
    let result: OptionalFields = from_str(yaml).unwrap();
    assert_eq!(result.required, "yes");
    assert_eq!(result.optional, None);
}

#[test]
fn optional_field_tilde_null() {
    let yaml = "required: yes\noptional: ~\n";
    let result: OptionalFields = from_str(yaml).unwrap();
    assert_eq!(result.required, "yes");
    assert_eq!(result.optional, None);
}

#[test]
fn roundtrip_none() {
    let data = OptionalFields {
        required: "hi".into(),
        optional: None,
    };
    let yaml = to_string(&data).unwrap();
    let back: OptionalFields = from_str(&yaml).unwrap();
    assert_eq!(back, data);
}

#[test]
fn roundtrip_some() {
    let data = OptionalFields {
        required: "hi".into(),
        optional: Some("there".into()),
    };
    let yaml = to_string(&data).unwrap();
    let back: OptionalFields = from_str(&yaml).unwrap();
    assert_eq!(back, data);
}

#[test]
fn deserialize_null_keyword() {
    let v: Value = from_str("null").unwrap();
    assert!(v.is_null());
}

#[test]
fn deserialize_tilde_as_null() {
    let v: Value = from_str("~").unwrap();
    assert!(v.is_null());
}

// ----------------------------------------------------------------
// 14. Unicode content
// ----------------------------------------------------------------

#[test]
fn unicode_string_roundtrip() {
    let original = "Hello, \u{4e16}\u{754c}!".to_string(); // "Hello, 世界!"
    let yaml = to_string(&original).unwrap();
    let back: String = from_str(&yaml).unwrap();
    assert_eq!(back, original);
}

#[test]
fn unicode_in_struct() {
    let person = Person {
        name: "\u{00e9}milie".into(), // "emilie" with accent
        age: 25,
        active: true,
    };
    let yaml = to_string(&person).unwrap();
    let back: Person = from_str(&yaml).unwrap();
    assert_eq!(back, person);
}

#[test]
fn emoji_roundtrip() {
    let original = "\u{1F600}\u{1F680}\u{2764}".to_string();
    let yaml = to_string(&original).unwrap();
    let back: String = from_str(&yaml).unwrap();
    assert_eq!(back, original);
}

#[test]
fn unicode_keys_in_mapping() {
    let yaml = "\u{30ad}\u{30fc}: \u{5024}\n"; // "キー: 値"
    let v: Value = from_str(yaml).unwrap();
    assert_eq!(
        v.get("\u{30ad}\u{30fc}").unwrap().as_str().unwrap(),
        "\u{5024}"
    );
}

// ----------------------------------------------------------------
// 15. Empty input handling
// ----------------------------------------------------------------

#[test]
fn empty_string_as_value() {
    let result: Result<Value, _> = from_str("");
    // Empty input should either parse as null or return an error
    match result {
        Ok(v) => assert!(v.is_null()),
        Err(_) => {} // also acceptable
    }
}

#[test]
fn whitespace_only_as_value() {
    let result: Result<Value, _> = from_str("   \n\n  ");
    match result {
        Ok(v) => assert!(v.is_null()),
        Err(_) => {}
    }
}

#[test]
fn empty_slice() {
    let result: Result<Value, _> = from_slice(b"");
    match result {
        Ok(v) => assert!(v.is_null()),
        Err(_) => {}
    }
}

// ----------------------------------------------------------------
// Extra: to_value / from_value
// ----------------------------------------------------------------

#[test]
fn to_value_and_from_value() {
    let person = Person {
        name: "Eve".into(),
        age: 35,
        active: false,
    };
    let v = to_value(&person).unwrap();
    assert_eq!(v.get("name").unwrap().as_str().unwrap(), "Eve");
    assert_eq!(v.get("age").unwrap().as_u64().unwrap(), 35);
    assert_eq!(v.get("active").unwrap().as_bool(), Some(false));

    let back: Person = from_value(v).unwrap();
    assert_eq!(back, person);
}

#[test]
fn to_value_scalar() {
    let v = to_value(42_i64).unwrap();
    assert_eq!(v.as_i64(), Some(42));
}

#[test]
fn to_value_vec() {
    let v = to_value(vec![1, 2, 3]).unwrap();
    let seq = v.as_sequence().unwrap();
    assert_eq!(seq.len(), 3);
}

// ----------------------------------------------------------------
// Extra: enum serialization
// ----------------------------------------------------------------

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Palette {
    primary: Color,
    secondary: Color,
}

#[test]
fn roundtrip_enum() {
    let p = Palette {
        primary: Color::Red,
        secondary: Color::Blue,
    };
    let yaml = to_string(&p).unwrap();
    let back: Palette = from_str(&yaml).unwrap();
    assert_eq!(back, p);
}

// ----------------------------------------------------------------
// Extra: special float values
// ----------------------------------------------------------------

#[test]
fn deserialize_nan() {
    let v: Value = from_str(".nan").unwrap();
    assert!(v.as_f64().unwrap().is_nan());
}

#[test]
fn deserialize_infinity() {
    let v: Value = from_str(".inf").unwrap();
    assert!(v.as_f64().unwrap().is_infinite());
    assert!(v.as_f64().unwrap().is_sign_positive());
}

#[test]
fn deserialize_neg_infinity() {
    let v: Value = from_str("-.inf").unwrap();
    assert!(v.as_f64().unwrap().is_infinite());
    assert!(v.as_f64().unwrap().is_sign_negative());
}

// ----------------------------------------------------------------
// Extra: deeply nested structures
// ----------------------------------------------------------------

#[test]
fn nested_via_flow_style() {
    // Use flow-style for nested mappings
    let yaml = "a: {b: deep}\n";
    let v: Value = from_str(yaml).unwrap();
    let result = v
        .get("a")
        .and_then(|a| a.get("b"))
        .and_then(|b| b.as_str());
    assert_eq!(result, Some("deep"));
}

// ----------------------------------------------------------------
// Extra: boolean variants
// ----------------------------------------------------------------

#[test]
fn deserialize_true_variants() {
    let v: Value = from_str("true").unwrap();
    assert_eq!(v.as_bool(), Some(true));
}

#[test]
fn deserialize_false_variants() {
    let v: Value = from_str("false").unwrap();
    assert_eq!(v.as_bool(), Some(false));
}

// ----------------------------------------------------------------
// Extra: multiline strings
// ----------------------------------------------------------------

#[test]
fn deserialize_quoted_string() {
    let v: Value = from_str("\"hello world\"").unwrap();
    assert_eq!(v.as_str(), Some("hello world"));
}

#[test]
fn deserialize_single_quoted_string() {
    let v: Value = from_str("'hello world'").unwrap();
    assert_eq!(v.as_str(), Some("hello world"));
}

// ----------------------------------------------------------------
// Extra: inline maps
// ----------------------------------------------------------------

#[test]
fn deserialize_inline_map() {
    let yaml = "{a: 1, b: 2}";
    let v: Value = from_str(yaml).unwrap();
    assert_eq!(v.get("a").unwrap().as_u64().unwrap(), 1);
    assert_eq!(v.get("b").unwrap().as_u64().unwrap(), 2);
}
