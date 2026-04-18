<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Deployment

SSG creates deploy configs for each platform. Use the `--deploy` flag to pick one.

## Supported Platforms

```sh
ssg --deploy netlify
ssg --deploy vercel
ssg --deploy cloudflare
ssg --deploy github
```

## Netlify

Creates a `netlify.toml` file in the site folder:

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

Creates a `vercel.json` file:

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

Creates `_headers` and `_redirects` files in the site folder:

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

Creates these files:

- `.nojekyll` — stops GitHub Pages from using Jekyll
- `CNAME` — custom domain file (if `base_url` has one)

Push the output folder to the `gh-pages` branch. GitHub Actions can do this in CI.

## Security Headers

All targets add these headers:

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

Skip `--deploy` and copy the output by hand. SSG makes plain HTML, CSS, and JS. No server code is needed.

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
- [SEO](seo.md) — Sitemaps and robots.txt
- [CLI Reference](cli.md) — `--deploy` flag details
