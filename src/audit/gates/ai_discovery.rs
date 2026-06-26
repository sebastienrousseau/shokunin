// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! AI-discovery protocol validation (depends on E8 — issue #552).
//!
//! Asserts presence + schema-validity of:
//! - `llms.txt` (already shipped in v0.0.43).
//! - `agents.txt` (E8).
//! - `.well-known/ai-plugin.json` (E8).
//!
//! When the E8-produced files are absent, the gate emits info-level
//! findings rather than errors — those upgrade to enforcement once
//! issue #552 (E8) merges.

use super::super::{AuditGate, AuditOptions, Finding, Severity, Site};

const NAME: &str = "ai_discovery";

/// AI-discovery protocol gate.
#[derive(Debug, Clone, Copy)]
pub struct AiDiscoveryGate;

impl AuditGate for AiDiscoveryGate {
    fn name(&self) -> &'static str {
        NAME
    }

    fn explain(&self) -> &'static str {
        "Asserts that AI-discovery protocol files exist + parse: \
         llms.txt (shipped in v0.0.43), agents.txt and \
         .well-known/ai-plugin.json (both from E8 — issue #552). \
         Files produced by E8 emit info notes when absent; the \
         already-shipped llms.txt emits a warning."
    }

    fn run(&self, site: &Site, _opts: &AuditOptions) -> Vec<Finding> {
        let mut findings = Vec::new();

        // llms.txt — already shipped; missing = warn
        let llms = site.root.join("llms.txt");
        if !llms.exists() {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Warn,
                    "llms.txt is missing (LlmManifestPlugin should emit it)",
                )
                .with_code("AI-LLMS-MISSING")
                .with_path("llms.txt".to_string()),
            );
        } else if let Ok(text) = std::fs::read_to_string(&llms) {
            if text.trim().is_empty() {
                findings.push(
                    Finding::new(NAME, Severity::Warn, "llms.txt is empty")
                        .with_code("AI-LLMS-EMPTY")
                        .with_path("llms.txt".to_string()),
                );
            }
        }

        // agents.txt — depends on E8
        let agents = site.root.join("agents.txt");
        if !agents.exists() {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Info,
                    "agents.txt absent (depends on E8 — issue #552)",
                )
                .with_code("AI-AGENTS-MISSING")
                .with_path("agents.txt".to_string()),
            );
        }

        // .well-known/ai-plugin.json — depends on E8
        let plugin_json = site.root.join(".well-known/ai-plugin.json");
        if !plugin_json.exists() {
            findings.push(
                Finding::new(
                    NAME,
                    Severity::Info,
                    ".well-known/ai-plugin.json absent (depends on E8 — issue #552)",
                )
                .with_code("AI-PLUGIN-JSON-MISSING")
                .with_path(".well-known/ai-plugin.json".to_string()),
            );
        } else if let Ok(text) = std::fs::read_to_string(&plugin_json) {
            if serde_json::from_str::<serde_json::Value>(&text).is_err() {
                findings.push(
                    Finding::new(
                        NAME,
                        Severity::Error,
                        ".well-known/ai-plugin.json is not valid JSON",
                    )
                    .with_code("AI-PLUGIN-JSON-INVALID")
                    .with_path(".well-known/ai-plugin.json".to_string()),
                );
            }
        }

        findings
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn site_with_files(files: &[(&str, &str)]) -> Site {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        for (name, body) in files {
            let p = root.join(name);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&p, body).unwrap();
        }
        std::mem::forget(tmp);
        Site {
            root,
            html_files: Vec::new(),
        }
    }

    #[test]
    fn all_present_and_valid_passes() {
        let s = site_with_files(&[
            ("llms.txt", "# llms\n"),
            ("agents.txt", "# agents\n"),
            (".well-known/ai-plugin.json", r#"{"schema_version":"v1"}"#),
        ]);
        let f = AiDiscoveryGate.run(&s, &AuditOptions::default());
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn missing_e8_files_emit_info_not_error() {
        let s = site_with_files(&[("llms.txt", "# llms\n")]);
        let f = AiDiscoveryGate.run(&s, &AuditOptions::default());
        assert!(f.iter().all(|x| matches!(x.severity, Severity::Info)));
        let codes: Vec<_> =
            f.iter().filter_map(|x| x.code.as_deref()).collect();
        assert!(codes.contains(&"AI-AGENTS-MISSING"));
        assert!(codes.contains(&"AI-PLUGIN-JSON-MISSING"));
    }

    #[test]
    fn invalid_ai_plugin_json_flagged_error() {
        let s = site_with_files(&[
            ("llms.txt", "# llms\n"),
            (".well-known/ai-plugin.json", "{ this is not json }"),
        ]);
        let f = AiDiscoveryGate.run(&s, &AuditOptions::default());
        assert!(f
            .iter()
            .any(|x| x.code.as_deref() == Some("AI-PLUGIN-JSON-INVALID")
                && matches!(x.severity, Severity::Error)));
    }

    #[test]
    fn missing_llms_txt_warns() {
        let s = site_with_files(&[
            ("agents.txt", "# agents\n"),
            (".well-known/ai-plugin.json", r#"{"schema_version":"v1"}"#),
        ]);
        let f = AiDiscoveryGate.run(&s, &AuditOptions::default());
        let llms = f
            .iter()
            .find(|x| x.code.as_deref() == Some("AI-LLMS-MISSING"))
            .expect("missing llms.txt finding");
        assert!(matches!(llms.severity, Severity::Warn));
    }

    #[test]
    fn empty_llms_txt_warns() {
        let s = site_with_files(&[
            ("llms.txt", "   \n   "),
            ("agents.txt", "# agents\n"),
            (".well-known/ai-plugin.json", r#"{"schema_version":"v1"}"#),
        ]);
        let f = AiDiscoveryGate.run(&s, &AuditOptions::default());
        let empty = f
            .iter()
            .find(|x| x.code.as_deref() == Some("AI-LLMS-EMPTY"))
            .expect("empty llms.txt finding");
        assert!(matches!(empty.severity, Severity::Warn));
    }

    #[test]
    fn all_three_missing_emits_warn_plus_two_infos() {
        let s = site_with_files(&[("placeholder.txt", "noop")]);
        let f = AiDiscoveryGate.run(&s, &AuditOptions::default());
        let codes: Vec<_> =
            f.iter().filter_map(|x| x.code.as_deref()).collect();
        assert!(codes.contains(&"AI-LLMS-MISSING"));
        assert!(codes.contains(&"AI-AGENTS-MISSING"));
        assert!(codes.contains(&"AI-PLUGIN-JSON-MISSING"));
        let warn_count = f
            .iter()
            .filter(|x| matches!(x.severity, Severity::Warn))
            .count();
        let info_count = f
            .iter()
            .filter(|x| matches!(x.severity, Severity::Info))
            .count();
        assert_eq!(warn_count, 1);
        assert_eq!(info_count, 2);
    }

    #[test]
    fn valid_ai_plugin_json_passes_silent() {
        let s = site_with_files(&[
            ("llms.txt", "# llms\n"),
            ("agents.txt", "# agents\n"),
            (".well-known/ai-plugin.json", r#"{"foo": [1, 2, 3]}"#),
        ]);
        let f = AiDiscoveryGate.run(&s, &AuditOptions::default());
        assert!(f.is_empty(), "got {f:?}");
    }

    #[test]
    fn metadata_methods_exposed() {
        let g = AiDiscoveryGate;
        assert_eq!(g.name(), "ai_discovery");
        assert!(g.explain().contains("llms.txt"));
        let _copy: AiDiscoveryGate = g;
        let _clone = g;
        assert!(format!("{g:?}").contains("AiDiscoveryGate"));
    }
}
