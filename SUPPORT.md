<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Support

Where to go, depending on what you need. Please use the right channel —
it is the difference between a fast answer and none.

## Reporting a security vulnerability

**Do not open a public issue.** Follow
[`SECURITY.md`](SECURITY.md), which describes the private reporting
channel and the 72-hour acknowledgement window.

## Something is broken

Open a [bug report][bug]. The template asks for the version, platform
and a reproduction, and all three matter: several bugs in this project
have been platform-specific, and a report without a reproduction usually
cannot be acted on.

Two things worth including that are easy to forget:

- **The exact command**, not a paraphrase. `ssg build` and
  `ssg -f config.toml` take different paths through the code.
- **Whether it reproduces from a clean checkout.** Stale `public/` or
  `.ssg-cache` output explains a surprising share of reports.

## Something is missing

Open a [feature request][feat]. Say what you are trying to achieve
rather than the API you imagine — the underlying goal is often reachable
today, and where it is not, knowing it shapes a better design.

## A question about usage

Open a [discussion][discuss] rather than an issue. Issues are for work
that will change the repository; questions that turn out to need a code
change get converted.

Before asking, the fastest answers are usually in:

| Question | Where |
|---|---|
| How do I configure X? | [`docs/guide/`](docs/guide/) |
| What does this flag do? | `ssg <command> --help`, or `man ssg` |
| How does the build actually work? | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) |
| Why was it built this way? | [`docs/adr/`](docs/adr/) |
| How do I work on SSG itself? | [`DEVELOPMENT.md`](DEVELOPMENT.md) |
| Is my slow build normal? | [`BENCHMARKS.md`](BENCHMARKS.md) |

## Contributing a change

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the workflow and
[`DEVELOPMENT.md`](DEVELOPMENT.md) for running every CI gate locally.
The table there maps each CI job to the exact command that reproduces
it, and it is checked against the workflow — so it will not send you
somewhere the pipeline does not go.

## What is supported

Supported versions are listed in [`SECURITY.md`](SECURITY.md). The
versioning policy — why this project is `0.0.x` and what that promises —
is [ADR-0009](docs/adr/0009-versioning-policy-0.0.x-until-0.0.999.md).

## Response expectations

This is a volunteer-maintained project. Security reports are
acknowledged within 72 hours. Everything else is best-effort, and a
well-formed report with a reproduction will always move faster than one
without.

[bug]: https://github.com/sebastienrousseau/static-site-generator/issues/new?template=bug_report.md
[feat]: https://github.com/sebastienrousseau/static-site-generator/issues/new?template=feature_request.md
[discuss]: https://github.com/sebastienrousseau/static-site-generator/discussions
