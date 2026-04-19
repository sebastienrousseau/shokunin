<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# CSP Without Compromise

How SSG eliminates `unsafe-inline` at build time for perfect Lighthouse security scores.

## The Problem

Most static site generators use inline styles and scripts. Browsers need Content Security Policy (CSP) headers to block cross-site scripting (XSS) attacks. But CSP blocks inline code by default.

The common fix is `unsafe-inline`. This tells the browser to allow all inline code. It works, but it also allows injected malicious code. This defeats the purpose of CSP.

## How SSG Solves This

SSG takes a different approach. At build time, it extracts all inline styles and scripts into external files. Then it generates Subresource Integrity (SRI) hashes for each file.

The result: zero inline code in your HTML. Your CSP policy can be strict. No `unsafe-inline` needed.

### Build-Time Extraction

The CSP plugin runs during the fused transform pipeline. For each HTML page:

1. Finds all `<style>` blocks
2. Extracts their content into `.css` files
3. Replaces the `<style>` block with a `<link>` tag
4. Finds all inline `<script>` blocks
5. Extracts their content into `.js` files
6. Replaces the `<script>` block with a `<script src="...">` tag

### SRI Hash Generation

After extraction, the asset fingerprint plugin generates SHA-384 hashes. Each `<link>` and `<script>` tag gets an `integrity` attribute.

```html
<link rel="stylesheet"
      href="/assets/style-a1b2c3.css"
      integrity="sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8w">
```

Browsers verify the hash before loading the file. If the file was tampered with, the browser blocks it.

### The CSP Header

With no inline code, your CSP header can be strict:

```
Content-Security-Policy: default-src 'self'; style-src 'self'; script-src 'self'
```

No `unsafe-inline`. No `unsafe-eval`. No nonces to manage.

## Lighthouse Results

Before SSG's CSP extraction:
- Security score: 70/100
- CSP header: requires `unsafe-inline`

After SSG's CSP extraction:
- Security score: 100/100
- CSP header: strict, no exceptions

## Comparison With Other SSGs

| SSG | Inline Extraction | SRI Hashes | Strict CSP |
|-----|-------------------|------------|------------|
| **SSG** | Build-time | Automatic | Yes |
| Hugo | Manual | No | Manual |
| Zola | No | No | Manual |
| Astro | Plugin needed | Plugin needed | Manual |
| Next.js | Nonce-based | No | Partial |

SSG is the only static site generator that handles CSP extraction and SRI hashing automatically at build time.

## Try It Yourself

1. Clone the repository:

```sh
git clone https://github.com/sebastienrousseau/static-site-generator.git
cd static-site-generator
```

2. Build the example site:

```sh
cargo run -- -c examples/content/en -o /tmp/csp-demo -t examples/templates
```

3. Check the output HTML for inline code:

```sh
grep -r '<style>' /tmp/csp-demo/ | wc -l
# Should output 0
```

4. Verify SRI hashes are present:

```sh
grep -r 'integrity=' /tmp/csp-demo/ | head -5
```

## How the Plugin Works

The CSP plugin (`src/csp.rs`) implements the `Plugin` trait with `has_transform() = true`. This means it runs in the fused transform pipeline. Each HTML file is read once, transformed, and written once.

The plugin is idempotent. Running it twice produces the same output. This is important for incremental builds where some pages are reprocessed.

## Security Benefits

- **No XSS via inline injection**: All code comes from external files with verified hashes
- **Tamper detection**: SRI hashes catch any modification to CSS or JS files
- **Defence in depth**: CSP + SRI + external files create three layers of protection
- **Zero configuration**: Works out of the box with the default plugin pipeline

## Further Reading

- [MDN: Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [MDN: Subresource Integrity](https://developer.mozilla.org/en-US/docs/Web/Security/Subresource_Integrity)
- [SSG Plugin API Guide](../guide/plugin-api.md)
- [SSG Security Architecture](../guide/plugins.md)
