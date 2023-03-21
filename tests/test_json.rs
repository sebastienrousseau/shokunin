#[cfg(test)]
mod tests {
    use serde_json::json;
    use serde_json::Map;
    use serde_json::Value;
    use ssg::json::{manifest, ManifestOptions};

    #[test]
    fn test_manifest_with_empty_options() {
        let options = ManifestOptions::default();
        let expected_result = r#"{
  "background_color": "",
  "description": "",
  "dir": "",
  "display": "",
  "icons": "",
  "identity": "",
  "lang": "",
  "name": "",
  "orientation": "",
  "scope": "",
  "short_name": "",
  "start_url": "",
  "theme_color": ""
}"#;
        let result = manifest(&options);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_manifest_with_non_empty_options() {
        let mut options = ssg::json::ManifestOptions {
            name: "My App".to_string(),
            short_name: "My App".to_string(),
            start_url: "/".to_string(),
            theme_color: "#ffffff".to_string(),
            ..Default::default()
        };
        options.name = "My App".to_string();
        options.short_name = "My App".to_string();
        options.start_url = "/".to_string();
        options.theme_color = "#ffffff".to_string();

        let mut expected_result = Map::new();
        expected_result
            .insert("background_color".to_string(), json!(""));
        expected_result.insert("description".to_string(), json!(""));
        expected_result.insert("dir".to_string(), json!(""));
        expected_result.insert("display".to_string(), json!(""));
        expected_result.insert("icons".to_string(), json!(""));
        expected_result.insert("identity".to_string(), json!(""));
        expected_result.insert("lang".to_string(), json!(""));
        expected_result.insert("name".to_string(), json!("My App"));
        expected_result.insert("orientation".to_string(), json!(""));
        expected_result.insert("scope".to_string(), json!(""));
        expected_result
            .insert("short_name".to_string(), json!("My App"));
        expected_result.insert("start_url".to_string(), json!("/"));
        expected_result
            .insert("theme_color".to_string(), json!("#ffffff"));

        let result = manifest(&options);
        assert_eq!(
            serde_json::from_str::<Map<String, Value>>(&result)
                .unwrap(),
            expected_result
        );
    }
}

// #[cfg(test)]
// mod tests {
//     use serde_json::Value;
//     use ssg::json::manifest;
//     use ssg::json::ManifestOptions;
//     use std::error::Error;

//     // fn test_manifest() -> Result<(), Box<dyn Error>> {
//     //     let expected_json = ManifestOptions {
//     //         background_color: "#ffffff".to_string(),
//     //         description: "test".to_string(),
//     //         dir: "/".to_string(),
//     //         display: "standalone".to_string(),
//     //         icons: "{ \"src\": \"icon/lowres.webp\", \"sizes\": \"64x64\", \"type\": \"image/webp\" }, { \"src\": \"icon/lowres.png\", \"sizes\": \"64x64\" }".to_string(),
//     //         identity: "/".to_string(),
//     //         lang: "en-US".to_string(),
//     //         name: "Test Web App".to_string(),
//     //         orientation: "any".to_string(),
//     //         scope: "/".to_string(),
//     //         short_name: "Test".to_string(),
//     //         start_url: "/index.html".to_string(),
//     //         theme_color: "#007bff".to_string(),
//     //     };

//     //     let options = ManifestOptions {
//     //         background_color: "#ffffff".to_string(),
//     //         description: "test".to_string(),
//     //         dir: "/".to_string(),
//     //         display: "standalone".to_string(),
//     //         icons: "{ \"src\": \"icon/lowres.webp\", \"sizes\": \"64x64\", \"type\": \"image/webp\" }, { \"src\": \"icon/lowres.png\", \"sizes\": \"64x64\" }".to_string(),
//     //         identity: "/".to_string(),
//     //         lang: "en-US".to_string(),
//     //         name: "Test Web App".to_string(),
//     //         orientation: "any".to_string(),
//     //         scope: "/".to_string(),
//     //         short_name: "Test".to_string(),
//     //         start_url: "/index.html".to_string(),
//     //         theme_color: "#007bff".to_string(),
//     //     };

//     //     let actual_json = manifest(&options);
//     //     let expected_value: Value =
//     //         serde_json::from_str(&expected_json)?;

//     //     // let expected_value: Value =
//     //     //     serde_json::from_str(&expected_json)?;
//     //     // let actual_value: Value = serde_json::from_str(&actual_json)?;

//     //     // assert_eq!(
//     //     //     actual_value, expected_value,
//     //     //     "Expected: {}, Actual: {}",
//     //     //     &expected_json, actual_json
//     //     // );

//     //     Ok(())
//     // }

//     #[test]
//     fn test_manifest_with_empty_values() -> Result<(), Box<dyn Error>> {
//         let expected_json = r#"{
//             "background_color": "",
//             "dir": "",
//             "description": "",
//             "display": "",
//             "icons": "",
//             "identity": "",
//             "lang": "",
//             "name": "",
//             "orientation": "",
//             "scope": "",
//             "start_url": "",
//             "short_name": "",
//             "theme_color": ""
//         }"#;

//         let actual_json = manifest(
//             "", "", "", "", "", "", "", "", "", "", "", "", "",
//         );

//         let expected_value: Value =
//             serde_json::from_str(expected_json)?;
//         let actual_value: Value = serde_json::from_str(&actual_json)?;

//         assert_eq!(
//             actual_value, expected_value,
//             "Expected: {}, Actual: {}",
//             expected_json, actual_json
//         );

//         Ok(())
//     }
// }
