<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Local LLM Content Pipelines for Static Sites

A technical reference for privacy-preserving AI content augmentation
with SSG.

## Abstract

This document describes SSG's local LLM integration — a build-time
content pipeline that uses Ollama or llama.cpp to auto-generate alt
text, meta descriptions, and structured data without sending content to
cloud APIs.

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  Markdown    │────▶│  SSG Build   │────▶│  HTML Output  │
│  Content     │     │  Pipeline    │     │  + AI Fields  │
└─────────────┘     └──────┬───────┘     └──────────────┘
                           │
                    ┌──────▼───────┐
                    │  Local LLM   │
                    │  (Ollama)    │
                    └──────────────┘
```

### Pipeline Stages

1. **Content compilation** — Markdown → HTML via staticdatagen
2. **AI readiness check** — `AiPlugin` validates alt text, generates
   `llms.txt` and `ai-provenance.json`
3. **LLM augmentation** — `LlmPlugin` invokes local model for:
   - Missing image alt text
   - Short/absent meta descriptions
   - JSON-LD description fields
4. **Post-processing** — SEO, fingerprinting, minification

### Privacy Model

- **Zero cloud dependency** — all inference runs on localhost
- **No data exfiltration** — content never leaves the build machine
- **Configurable** — model, endpoint, and prompts set in `ssg.toml`
- **Opt-in** — LLM features activate only when Ollama is reachable
- **Dry-run** — `--ai-dry-run` previews generated text without writing
- **Provenance tracking** — `ai-provenance.json` documents which
  fields are AI-generated vs human-authored

## Configuration

```toml
[ai]
model = "llama3"
endpoint = "http://localhost:11434"
```

## CLI Flags

| Flag | Description |
|------|-------------|
| `--ai-dry-run` | Print generated text without writing files |

## Output Files

| File | Purpose |
|------|---------|
| `llms.txt` | AI crawler guidance (like robots.txt for LLMs) |
| `llms-full.txt` | Full content index with titles and snippets |
| `ai-provenance.json` | Content provenance log |

## Benchmarks

Performance impact of the LLM plugin depends on model size and
hardware. With Ollama running `llama3` on Apple Silicon:

- Alt text generation: ~200ms per image
- Meta description: ~500ms per page
- Full build with 100 pages: adds ~15s to build time

The plugin is designed to be skipped entirely when no LLM is
available, adding zero overhead to non-AI builds.

## References

- [llms.txt specification](https://llmstxt.org/)
- [Ollama API documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [SSG AI Plugin source](https://github.com/sebastienrousseau/static-site-generator/blob/main/src/ai.rs)
- [SSG LLM Plugin source](https://github.com/sebastienrousseau/static-site-generator/blob/main/src/llm.rs)
