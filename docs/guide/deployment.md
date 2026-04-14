<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Deployment

SSG generates platform-specific deployment configuration with the `--deploy` flag.

## Supported Platforms

```sh
ssg --deploy netlify
ssg --deploy vercel
ssg --deploy cloudflare
ssg --deploy github
```

## Netlify

Generates `netlify.toml` in the site directory:

```toml
[build]
  publish = "public"
  command = "ssg -c content -o public -t templates"

[[headers]]
  for = "/*"
  [headers.values]
    X-Content-Type-Options = "nosniff"
    X-Frame-Options = "DENY"
    Strict-Transport-Security = "max-age=31536000; includeSubDomains"
    Content-Security-Policy = "default-src 'self'; ..."
```

## Vercel

Generates `vercel.json`:

```json
{
  "buildCommand": "ssg -c content -o public -t templates",
  "outputDirectory": "public",
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "X-Content-Type-Options", "value": "nosniff" },
        { "key": "X-Frame-Options", "value": "DENY" }
      ]
    }
  ]
}
```

## Cloudflare Pages

Generates `_headers` and `_redirects` files in the site directory:

```
/*
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
  X-XSS-Protection: 1; mode=block
  Referrer-Policy: strict-origin-when-cross-origin
  Strict-Transport-Security: max-age=31536000; includeSubDomains
  Content-Security-Policy: default-src 'self'; ...
  Permissions-Policy: camera=(), microphone=(), geolocation=()
```

## GitHub Pages

Generates:

- `.nojekyll` — prevents GitHub Pages from processing with Jekyll
- `CNAME` — custom domain file (if `base_url` has a custom domain)

Deploy with GitHub Actions by pushing the output directory to the `gh-pages` branch.

## Security Headers

All deployment targets include these security headers:

| Header | Value |
| :--- | :--- |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `X-XSS-Protection` | `1; mode=block` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | `camera=(), microphone=(), geolocation=()` |
| `Content-Security-Policy` | `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' https: data:; font-src 'self' https:; connect-src 'self'; frame-ancestors 'none'` |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` |

## Manual Deployment

If you are not using `--deploy`, copy the output directory to any static file host. SSG output is plain HTML, CSS, and JavaScript with no server-side requirements.

## CI/CD Example

```yaml
# GitHub Actions
- name: Build site
  run: |
    cargo install ssg
    ssg -c content -o public -t templates

- name: Deploy to GitHub Pages
  uses: peaceiris/actions-gh-pages@v4
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    publish_dir: ./public
```

## Next Steps

- [Configuration](configuration.md) — `base_url` and other settings
- [SEO](seo.md) — sitemaps and robots.txt for production
- [CLI Reference](cli.md) — `--deploy` flag details
