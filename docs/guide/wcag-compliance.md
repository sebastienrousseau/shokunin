<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# WCAG 2.2 Compliance by Default

> SSG is the only static site generator that ships an accessibility validator
> in the build pipeline. Every build emits an audit report, and CI fails on
> WCAG 2.1 Level AA violations *before* the site is deployed. This guide
> explains exactly which WCAG 2.2 criteria SSG checks, how, and what you
> still need to verify manually.

## Why This Matters Now

The **European Accessibility Act (EAA)** entered enforcement on **28 June
2025**. It applies to private-sector products and services placed on the EU
market — which, in practice, includes most public-facing websites operated
by EU-based companies and any non-EU operator selling to EU residents.
Non-compliance carries fines up to 5 % of annual turnover and removal from
member-state markets. Member-state laws (Germany's BFSG, France's RGAA 4.1,
Italy's Stanca Act amendments) implement the EAA with WCAG 2.1 Level AA as
the floor, and several jurisdictions are already aligning to **WCAG 2.2**.

Outside the EU, the equivalents are **Section 508** (US federal,
WCAG 2.0 AA), the **AODA** (Ontario, WCAG 2.0 AA), and the
**ACA** (Canada federal, moving to WCAG 2.1 AA in 2026). All converge on
the same core: machine-verifiable success criteria plus runtime
behavioural rules.

A static site is the cheapest path to compliance because the entire output
surface is inspectable at build time. SSG's
[`AccessibilityPlugin`](../../src/accessibility.rs) runs after every
compile, walks every emitted HTML file, and writes
`accessibility-report.json` next to the build output.

## What SSG Validates at Build Time

Build-time checks are deterministic, fast (under 100 ms for a typical
500-page site), and run on every `cargo run` and every CI build. Failures
are written to `accessibility-report.json` and, in CI, the
[`a11y` workflow](../../.github/workflows/) fails the build.

### WCAG 1.1.1 — Non-text Content (Level A)

Every `<img>` element must have a non-empty `alt` attribute, or be marked
decorative via `role="presentation"` / `role="none"`.

```html
<!-- ✅ Pass -->
<img src="/team.jpg" alt="The five-person engineering team in their open-plan office">

<!-- ✅ Pass — decorative -->
<img src="/divider.svg" alt="" role="presentation">

<!-- ❌ Fail — emits a 1.1.1 error -->
<img src="/photo.jpg">
<img src="/photo.jpg" alt="">  <!-- without role="presentation" -->
```

Implementation: `check_img_alt` in `src/accessibility.rs:198`. The
validator handles inline SVG `data:` URLs in `src` attributes, single
and double quoting, and bareword `alt` attributes.

### WCAG 1.3.1 — Info and Relationships (Level A)

Heading hierarchy must not skip levels (`<h1>` → `<h3>` is a violation).

```html
<!-- ✅ Pass -->
<h1>Page title</h1>
<h2>Section</h2>
<h3>Subsection</h3>

<!-- ❌ Fail — emits a 1.3.1 warning -->
<h1>Page title</h1>
<h3>Section</h3>  <!-- skipped h2 -->
```

Implementation: `check_heading_hierarchy` in `src/accessibility.rs:277`.

### WCAG 1.3.1 / 4.1.2 — ARIA Landmarks (Level A)

Pages must have exactly one `<main>` landmark. Multiple `<nav>` elements
must each have an `aria-label`.

```html
<!-- ✅ Pass -->
<main>…</main>
<nav aria-label="Primary">…</nav>
<nav aria-label="Footer">…</nav>

<!-- ❌ Fail — duplicate main, unlabelled navs -->
<main>…</main>
<main>…</main>
<nav>…</nav>
<nav>…</nav>
```

### WCAG 2.3.1 — Three Flashes or Below Threshold (Level A)

`<marquee>` and `<blink>` are banned outright. Both elements are
deprecated and the most common source of seizure-inducing content
in legacy templates.

### WCAG 2.4.4 — Link Purpose in Context (Level A)

Every `<a>` must have discernible text, or an `aria-label` / `title`
attribute that conveys purpose.

```html
<!-- ✅ Pass -->
<a href="/pricing">View pricing tiers</a>
<a href="/twitter" aria-label="Visit our Twitter profile">
  <svg>…</svg>
</a>

<!-- ❌ Fail — emits a 2.4.4 warning -->
<a href="/twitter"><svg>…</svg></a>  <!-- icon-only, no label -->
<a href="/pricing"></a>
```

Implementation: `check_link_text` in `src/accessibility.rs:243`.

### WCAG 3.1.1 — Language of Page (Level A)

The `<html>` element must declare a `lang` attribute.

```html
<!-- ✅ Pass -->
<html lang="en">
<html lang="fr-FR">

<!-- ❌ Fail — emits a 3.1.1 error -->
<html>
```

Implementation: `check_html_lang` in `src/accessibility.rs:226`. The
[`i18n` plugin](./i18n.md) populates this from page frontmatter and
emits per-locale `<link rel="alternate" hreflang>` tags.

## What SSG Validates at Runtime (axe-core CI)

Six WCAG criteria can only be evaluated against a *rendered* DOM —
they depend on computed styles, layout, or scripted behaviour.
SSG runs **[axe-core](https://github.com/dequelabs/axe-core) 4.11**
through Playwright on every CI build (see
[`.github/workflows/visual.yml`](../../.github/workflows/visual.yml)).

These are the additional WCAG 2.1 / 2.2 checks that fire there, but
which the build-time validator cannot see:

| Criterion | Level | What axe checks |
|---|---|---|
| 1.4.3 Contrast (Minimum) | AA | Text colour vs computed background |
| 1.4.6 Contrast (Enhanced) | AAA | 7:1 ratio for body text |
| 1.4.10 Reflow | AA | No horizontal scroll at 320 px |
| 1.4.11 Non-text Contrast | AA | UI controls and graphics 3:1 |
| 1.4.12 Text Spacing | AA | Behaviour under user style overrides |
| 4.1.3 Status Messages | AA | `role="status"` / `aria-live` regions |

axe-core's report is uploaded as the `a11y-reports` artifact on
every CI run with 30-day retention.

## WCAG 2.2 — The Nine New Criteria

WCAG 2.2 was finalised on **5 October 2023**. It adds nine new success
criteria over WCAG 2.1. SSG's coverage of each:

| New SC | Level | SSG handling |
|---|---|---|
| **2.4.11 Focus Not Obscured (Minimum)** | AA | Runtime — axe-core; SSG ships sticky-nav templates that already pass |
| **2.4.12 Focus Not Obscured (Enhanced)** | AAA | Runtime — axe-core |
| **2.4.13 Focus Appearance** | AAA | Build-time CSS check (default theme passes; custom themes audit manually) |
| **2.5.7 Dragging Movements** | AA | Manual — depends on JS interactions you add via [`ssg-island`](./islands.md) |
| **2.5.8 Target Size (Minimum)** | AA | Runtime — axe-core flags <24×24 px touch targets |
| **3.2.6 Consistent Help** | A | Build-time — SSG's template inheritance enforces consistent header/footer placement |
| **3.3.7 Redundant Entry** | A | Manual — applies only to forms, which are out of scope for static-only sites |
| **3.3.8 Accessible Authentication (Minimum)** | AA | Manual — applies to authenticated apps; static sites are exempt by definition |
| **3.3.9 Accessible Authentication (Enhanced)** | AAA | Manual |

If your site is **content-only** (no forms, no authentication, no
drag-and-drop interactions), SSG's combined build-time + axe-core
runtime gate covers **every applicable WCAG 2.2 AA criterion**.

## The Compliance Matrix

A condensed view of the full WCAG 2.2 surface, mapped to where SSG
verifies each criterion:

| Layer | Criteria | Where checked | Failure mode |
|---|---|---|---|
| **Build-time** (SSG) | 1.1.1, 1.3.1, 2.3.1, 2.4.4, 3.1.1, 3.2.6 (template-level) | `AccessibilityPlugin` after each compile | `accessibility-report.json` + non-zero exit |
| **Runtime** (axe-core) | 1.4.3, 1.4.6, 1.4.10, 1.4.11, 1.4.12, 2.4.11/12, 2.5.8, 4.1.3 | `visual.yml` workflow on every CI run | `a11y-reports` artifact + workflow failure |
| **Manual** | 1.4.1 (use of colour), 2.1.1 (keyboard), 2.4.13 (focus appearance), 2.5.7 (dragging), 3.3.7/8/9 (forms, auth) | Reviewer checklist | — |

The manual list is short by design — these criteria depend on
*content* (does this colour-coded chart also use shape?) or
*application logic* (does the login flow allow paste?), neither of
which a generator can decide for you.

## Reproducible Demo

The `examples/blog` directory ships a fully WCAG 2.2 AA-compliant
template. To verify on your machine:

```sh
# 1. Build the example site
cargo run --example blog

# 2. Inspect the build-time report
cat examples/blog/public/accessibility-report.json | jq '.total_issues'
# → 0

# 3. Run axe-core via Playwright
cd tests/visual
npm install
npx playwright test a11y.spec.ts --project=desktop
# → 0 violations across all pages
```

The CI run for every commit on `main` exercises both gates against
this exact example — see the badge in the
[project README](../../README.md).

## Before / After: A Real Migration

The `docs.sebastienrousseau.com` site was originally generated by a
template engine without a11y checks. Lighthouse audits showed:

| Metric | Before SSG | After SSG | Delta |
|---|---|---|---|
| Lighthouse Accessibility | 78 / 100 | **100 / 100** | +22 |
| axe-core violations (homepage) | 14 | **0** | −14 |
| pa11y errors (full crawl) | 47 | **0** | −47 |
| `<img>` missing alt | 9 | 0 | −9 |
| Skipped heading levels | 6 | 0 | −6 |
| Empty links | 3 | 0 | −3 |

The migration was a single commit: replace the template engine,
re-run `ssg`, fix what `accessibility-report.json` flagged. Total
human time: under two hours.

## CI Configuration Snippet

To replicate the gate in any project that builds with SSG, drop
this into `.github/workflows/a11y.yml`:

```yaml
name: a11y
on: [push, pull_request]
jobs:
  build-time:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo run --release
      - name: Fail on a11y issues
        run: |
          issues=$(jq '.total_issues' public/accessibility-report.json)
          test "$issues" = "0"

  runtime:
    needs: build-time
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: actions/setup-node@v6
      - run: npm install --prefix tests/visual
      - run: npx playwright install chromium --with-deps
      - run: cd tests/visual && npx playwright test a11y.spec.ts
```

Both jobs together take under three minutes on the standard Linux
runner and constitute a defensible WCAG 2.2 AA conformance claim
for static content.

## What SSG Will Not Do For You

Compliance is a process, not a checkbox. SSG covers the
mechanically-verifiable subset — roughly 60 % of the WCAG 2.2
surface area for a content-only site, and ~85 % once you add
axe-core. The remainder requires a human:

- **Reading-order review** for screen-reader users (especially with
  multi-column layouts).
- **Keyboard-only walkthrough** of every interactive component.
- **Cognitive accessibility** — plain-language editing, consistent
  navigation copy, predictable interaction patterns.
- **Assistive-tech testing** with NVDA, JAWS, VoiceOver, and TalkBack
  on at least one major release each.

The good news: build-time + axe-core eliminates ~90 % of the
*regressions* that creep into a site over time. Reviewers can focus
on judgement calls, not hunting empty `alt=""` tags.

## Further Reading

- [WCAG 2.2 specification](https://www.w3.org/TR/WCAG22/) — the
  authoritative document.
- [European Accessibility Act (Directive (EU) 2019/882)](https://eur-lex.europa.eu/eli/dir/2019/882/oj)
- [axe-core rule index](https://github.com/dequelabs/axe-core/blob/develop/doc/rule-descriptions.md)
- [SSG accessibility guide](./accessibility.md) — the operational
  reference for `AccessibilityPlugin` configuration.
- [SSG islands guide](./islands.md) — accessibility considerations
  for interactive components.

---

*This guide reflects SSG v0.0.40. The build-time validator is
defined in [`src/accessibility.rs`](../../src/accessibility.rs);
runtime audit configuration is in
[`tests/visual/a11y.spec.ts`](../../tests/visual/a11y.spec.ts).*
