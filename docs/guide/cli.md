<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# CLI Reference

Complete reference for the `ssg` command-line interface.

## Usage

```
ssg [OPTIONS]
```

## Options

### Project Setup

| Flag | Short | Value | Description |
| :--- | :--- | :--- | :--- |
| `--new` | `-n` | `NAME` | Create a new project with scaffolded content, templates, and config |
| `--config` | `-f` | `FILE` | Path to `ssg.toml` configuration file |

### Build Directories

| Flag | Short | Value | Description |
| :--- | :--- | :--- | :--- |
| `--content` | `-c` | `DIR` | Content directory (Markdown files) |
| `--output` | `-o` | `DIR` | Output directory for generated site |
| `--template` | `-t` | `DIR` | Template directory (Tera templates) |

### Development

| Flag | Short | Value | Description |
| :--- | :--- | :--- | :--- |
| `--serve` | `-s` | `DIR` | Start dev server serving the given directory |
| `--watch` | `-w` | | Watch for file changes and rebuild automatically |
| `--drafts` | | | Include draft pages in the build |

### Build Control

| Flag | Short | Value | Description |
| :--- | :--- | :--- | :--- |
| `--jobs` | `-j` | `N` | Number of Rayon parallel threads (default: number of CPUs) |
| `--validate` | | | Validate content schemas without building |
| `--deploy` | | `TARGET` | Generate deployment config: `netlify`, `vercel`, `cloudflare`, `github` |

### Output Control

| Flag | Short | Description |
| :--- | :--- | :--- |
| `--quiet` | `-q` | Suppress non-error output |
| `--verbose` | | Show detailed build information |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

## Environment Variables

| Variable | Default | Description |
| :--- | :--- | :--- |
| `SSG_HOST` | `127.0.0.1` | Dev server bind address. Use `0.0.0.0` for WSL2 / Codespaces |
| `SSG_PORT` | `3000` | Dev server port |

## Examples

### Scaffold a new project

```sh
ssg -n myblog -c content -o build -t templates
```

### Build a site

```sh
ssg -c content -o public -t templates
```

### Build with config file

```sh
ssg -f ssg.toml
```

### Dev server with watch

```sh
ssg -c content -o public -t templates -s public --watch
```

### Validate schemas only

```sh
ssg --validate -c content
```

### Build with drafts

```sh
ssg -c content -o public -t templates --drafts
```

### Generate Netlify config

```sh
ssg --deploy netlify
```

### Limit parallelism

```sh
ssg -c content -o public -t templates -j 4
```

### Quiet build

```sh
ssg -c content -o public -t templates -q
```

### Verbose build

```sh
ssg -c content -o public -t templates --verbose
```

### WSL2 / Codespaces dev server

```sh
SSG_HOST=0.0.0.0 ssg -c content -o public -t templates -s public
```

## Exit Codes

| Code | Meaning |
| :--- | :--- |
| `0` | Success |
| `1` | General error (invalid arguments, build failure, I/O error) |

## Config File vs CLI

CLI flags override values from `ssg.toml`. You can combine both:

```sh
ssg -f ssg.toml -o dist   # uses config but overrides output to "dist"
```

## Next Steps

- [Configuration](configuration.md) — full `ssg.toml` reference
- [Quick Start](quick-start.md) — get started with common commands
- [Deployment](deployment.md) — `--deploy` target details
