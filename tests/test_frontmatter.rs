// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ssg_frontmatter::{
        extract, extract_front_matter_str, extract_json_object_str,
        parse_json_object, parse_toml_table, parse_yaml_document,
        parse_yaml_hash,
    };
    use yaml_rust2::YamlLoader;

    #[test]
    fn test_extract_with_valid_content() {
        let content = "---\ntitle: Hello World\nauthor: John Doe\n---\nHello, world!";
        let result = extract(content);
        let expected: HashMap<String, String> = [
            ("title".to_string(), "Hello World".to_string()),
            ("author".to_string(), "John Doe".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_invalid_content() {
        let content = "Hello, world!";
        let result = extract(content);
        let expected: HashMap<String, String> = HashMap::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_empty_content() {
        let content = "";
        let result = extract(content);
        let expected: HashMap<String, String> = HashMap::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_empty_frontmatter() {
        let content = "";
        let result = extract(content);
        let expected: HashMap<String, String> = HashMap::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_with_toml_frontmatter() {
        let content = "+++\ntitle = \"Hello World\"\nauthor = \"John Doe\"\n+++\nHello, world!";
        let result = extract(content);
        let expected: HashMap<String, String> = [
            ("title".to_string(), "Hello World".to_string()),
            ("author".to_string(), "John Doe".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_front_matter_str() {
        let content = "---\nfoo: bar\nbaz: qux\n---\nHello, world!";
        assert_eq!(
            extract_front_matter_str(content, "---\n", "\n---\n"),
            Some("foo: bar\nbaz: qux")
        );

        let content =
            "+++\nfoo = \"bar\"\nbaz = \"qux\"\n+++\nHello, world!";
        assert_eq!(
            extract_front_matter_str(content, "+++\n", "\n+++\n"),
            Some("foo = \"bar\"\nbaz = \"qux\"")
        );

        let content = "{\n\"frontmatter\": {\n\"foo\": \"bar\",\n\"baz\": \"qux\"\n},\n\"content\": \"Hello, world!\"\n}";
        assert_eq!(
            extract_front_matter_str(content, "{", "}"),
            Some("\n\"frontmatter\": {\n\"foo\": \"bar\",\n\"baz\": \"qux\"\n")
        );

        let content = "Hello, world!";
        assert_eq!(
            extract_front_matter_str(content, "---\n", "\n---\n"),
            None
        );
        assert_eq!(
            extract_front_matter_str(content, "+++\n", "\n+++\n"),
            None
        );
        assert_eq!(extract_front_matter_str(content, "{", "}"), None);
    }

    #[test]
    fn test_parse_yaml_document() {
        let input = r#"
            foo: bar
            baz: qux
        "#;
        let expected_output = yaml_rust2::Yaml::Hash(
            vec![
                (
                    yaml_rust2::Yaml::String("foo".to_string()),
                    yaml_rust2::Yaml::String("bar".to_string()),
                ),
                (
                    yaml_rust2::Yaml::String("baz".to_string()),
                    yaml_rust2::Yaml::String("qux".to_string()),
                ),
            ]
            .into_iter()
            .collect(),
        );
        assert_eq!(
            parse_yaml_document(input).unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_parse_yaml_hash() {
        let yaml_str = r#"
    name: John Doe
    age: 42
    email: john.doe@example.com
"#;
        let docs = YamlLoader::load_from_str(yaml_str).unwrap();
        let yaml_hash = docs[0].as_hash().unwrap();
        let result = parse_yaml_hash(yaml_hash);
        let expected: HashMap<String, String> = vec![
            ("name".to_owned(), "John Doe".to_owned()),
            ("email".to_owned(), "john.doe@example.com".to_owned()),
        ]
        .into_iter()
        .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_yaml_hash_with_non_string_values() {
        let yaml_str = r#"
            name: John Doe
            age: 42
        "#;
        let docs = YamlLoader::load_from_str(yaml_str).unwrap();
        let yaml_hash = docs[0].as_hash().unwrap();
        let result = parse_yaml_hash(yaml_hash);
        let expected: HashMap<String, String> =
            [("name".to_owned(), "John Doe".to_owned())]
                .into_iter()
                .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_toml_table() {
        let toml_str = r#"
        [author]
        name = "John Doe"
        email = "john.doe@example.com"
        age = "42"
    "#;
        let toml: toml::Value = toml::from_str(toml_str).unwrap();
        let result = parse_toml_table(
            toml.get("author").unwrap().as_table().unwrap(),
        );
        let expected: HashMap<String, String> = vec![
            ("name".to_owned(), "John Doe".to_owned()),
            ("email".to_owned(), "john.doe@example.com".to_owned()),
            ("age".to_owned(), "42".to_owned()),
        ]
        .into_iter()
        .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_json_object_str() {
        let content1 = r#"{"name": "John Doe", "age": 42}"#;
        assert_eq!(
            extract_json_object_str(content1),
            Some(r#"{"name": "John Doe", "age": 42}"#)
        );
    }

    #[test]
    fn test_parse_json_object() {
        let json_str = r#"{
        "name": "John Doe",
        "age": 42,
        "email": "john.doe@example.com",
        "address": {
            "street": "123 Main St",
            "city": "Anytown",
            "state": "CA",
            "zip": "12345"
        }
    }"#;
        let json_value: serde_json::Value =
            serde_json::from_str(json_str).unwrap();
        let json_object = json_value.as_object().unwrap();

        let result = parse_json_object(json_object);
        let expected: HashMap<String, String> = vec![
        ("name".to_owned(), "John Doe".to_owned()),
        ("age".to_owned(), "42".to_owned()),
        ("email".to_owned(), "john.doe@example.com".to_owned()),
        (
            "address".to_owned(),
            r#"{"city":"Anytown","state":"CA","street":"123 Main St","zip":"12345"}"#.to_owned(),
        ),
    ]
    .into_iter()
    .collect();
        assert_eq!(result, expected);
    }
}
