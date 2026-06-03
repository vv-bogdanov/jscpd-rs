# Changelog

## Unreleased

### Changed

- Strip release binaries and abort on panic in release builds to reduce native
  npm prebuilt package size without changing runtime behavior.

## 0.1.11 - 2026-06-03

### Changed

- Add the OpenSSF Best Practices Passing badge to the README after the project
  reached Passing level.
- Remove internal helper-agent workflow notes, historical cloning notes, and
  duplicate upstream issue drafts from the public repository documentation.
- Update OpenSSF Best Practices evidence documentation to reflect the Passing
  status and the honest `N/A` handling for cryptography and memory-unsafe
  dynamic-analysis criteria.

## 0.1.10 - 2026-06-03

### Changed

- Refresh public npm/GitHub README onboarding with npm-first install guidance,
  absolute documentation links that work from npm, and the current public
  benchmark baseline.
- Update release and benchmark documentation so current evidence points at the
  GitHub Release automation path instead of stale pre-0.1.9 commits.
- Narrow generated npm platform-package keywords so prebuilt binary packages
  do not compete with the main `jscpd-rs` package in package search.
- Update the docs.rs crate root metadata for the current release line.

### Validation

- Recorded public benchmark baseline for this release:

| Case | Commit | Format | Rust avg | Upstream avg | Speedup | Compat |
| --- | --- | --- | ---: | ---: | ---: | --- |
| React | `f0dfee3` | JavaScript | 0.197325s | 10.413453s | 52.77x | pass |
| Next.js | `2bbb67b9` | TypeScript | 0.270786s | 14.983243s | 55.33x | pass |
| Prometheus | `a0524ee` | Go | 0.083162s | 4.842499s | 58.23x | pass |

## 0.1.9 - 2026-06-03

### Changed

- Refresh the npm README with the self-updating Socket `/latest` badge URL.
- Keep npm publication strict on registry integrity, signatures, and
  provenance, while treating fresh Socket `pendingScan` results as
  non-blocking during the post-publish window.
- Add a dedicated scheduled/manual Socket score workflow for strict
  post-indexing score enforcement.
- Make the server compatibility harness allocate free ports dynamically instead
  of relying on fixed local ports.

## 0.1.8 - 2026-06-03

### Changed

- Publish a clean patch release through the GitHub Release automation so npm
  provenance for the latest package version points at a retained release source.
- Include the README badge grouping and npm publish rerun fixes that landed
  after the `0.1.7` publication.

## 0.1.7 - 2026-06-02

### Changed

- Reduce the Rust supply-chain surface by reverting the direct `getrandom`
  dependency from 0.4 to the stable 0.2 line used before 0.1.5. This removes
  the extra WASI/WIT transitive dependency tail added by the major update while
  keeping OS-backed MCP session IDs.
- Configure Dependabot to keep grouped Cargo dependency updates to minor and
  patch releases, and explicitly ignore future major `getrandom` bumps unless
  a security advisory or concrete platform need justifies the larger dependency
  graph.

## 0.1.6 - 2026-06-02

### Added

- Add repo ownership and supply-chain maintenance signals: `CODEOWNERS`,
  OpenSSF Scorecard workflow and badge, root `.editorconfig`, and a project
  code of conduct.
- Add release-gate checks for GitHub Actions syntax through `actionlint` and
  Rust dependency policy through `cargo-deny`.
- Add post-publication npm registry checks for package integrity, registry
  signatures, SLSA provenance attestations, and `npm audit signatures`.
- Add Socket package score regression checks for the main npm package and
  prebuilt platform packages.

### Changed

- Improve prebuilt npm platform package metadata and README supply-chain notes.
- Remove the npm `test:npm-package` script from published package metadata; the
  script referenced repository-only files that are intentionally excluded from
  npm tarballs.

## 0.1.5 - 2026-06-02

### Changed

- Update GitHub Actions dependencies used by CI and release automation.
- Update `getrandom` to 0.4 and adapt MCP session ID generation to the new
  `getrandom::fill` API.

### Validation

- Passed local `scripts/prepublish-check.sh` and GitHub release-candidate
  publish gates on commit `06c801c`.
- Recorded public benchmark baseline for this release:

| Case | Commit | Format | Rust avg | Upstream avg | Speedup | Compat |
| --- | --- | --- | ---: | ---: | ---: | --- |
| React | `f0dfee3` | JavaScript | 0.193443s | 9.979393s | 51.59x | pass |
| Next.js | `2bbb67b9` | TypeScript | 0.281938s | 14.182806s | 50.30x | pass |
| Prometheus | `a0524ee` | Go | 0.086096s | 4.608737s | 53.53x | pass |

## 0.1.4 - 2026-06-02

### Changed

- Remove npm install-time builds: the main npm package no longer declares a
  `postinstall` lifecycle script and no longer invokes Cargo during install.
- Shrink the main npm package to runtime shim files, project metadata, README,
  license, changelog, and security/contributing docs.
- Keep npm runtime prebuilt-first: platform packages provide native binaries;
  unsupported npm platforms should install through Cargo.
- Add `SECURITY.md`, `CONTRIBUTING.md`, and Dependabot configuration for
  clearer supply-chain and maintenance signals.
- Update npm package checks to validate prebuilt-only behavior and local
  platform-package smoke tests.

## 0.1.3 - 2026-06-01

### Changed

- Tighten GitHub Release publication gates: npm and crates.io publication now
  run the release-candidate gate before publishing.
- Block the main npm package publication if any configured prebuilt platform
  package is missing or failed to publish.
- Add a core coverage gate to the release-candidate flow.
- Add an advisory server benchmark for comparing native and upstream
  `/api/check` latency.
- Refresh npm, prebuilt-binary, release-readiness, and README documentation for
  the prebuilt-first install path.

## 0.1.2 - 2026-06-01

### Changed

- Rename the Windows prebuilt npm package to `jscpd-rs-win` to avoid npm
  registry spam-policy false positives on the previous machine-generated name.
- Move the Linux arm64 prebuilt build to the Ubuntu 22.04 ARM runner for a more
  stable native ARM publication path and older glibc baseline.
- Allow npm release workflow reruns for a single prebuilt target without
  republishing the already-published main package.

## 0.1.1 - 2026-06-01

### Added

- npm prebuilt binary distribution wiring for Linux x64 GNU, Linux arm64 GNU,
  macOS x64, macOS arm64, and Windows x64 MSVC platform packages.
- Runtime npm shim resolution that prefers installed prebuilt optional
  packages and falls back to the existing Cargo source-build path.
- npm package checks for platform package metadata, source-build fallback, and
  local prebuilt-package smoke tests.
- GitHub Release npm publishing workflow that builds platform packages before
  publishing the main `jscpd-rs` package.

## 0.1.0 - 2026-05-31

First release candidate for `jscpd-rs`, a native Rust clone of upstream
`jscpd`.

### Added

- Native `jscpd` CLI binary with upstream-compatible command name and help
  shape.
- Native `jscpd-server` binary exposing `/`, `/api/health`, `/api/stats`,
  `/api/check`, `/api/recheck`, and `/mcp`.
- Coverage-first compatibility gates against the upstream `jscpd` submodule.
- CLI/config support for the main upstream option surface, including Commander
  edge cases covered by compatibility scripts.
- Native file discovery with `.gitignore`, global Git excludes, symlink policy,
  shebang detection, max size, max line, custom extension, and custom filename
  handling.
- Upstream-synchronized format registry with 223 formats and 206 extension
  mappings.
- Native Oxc-backed JavaScript, TypeScript, JSX, and TSX token processing.
- Native generic tokenization for long-tail formats, plus block handling for
  Markdown, markup, Vue, Svelte, Astro, Apex, and TAP where needed for current
  coverage gates.
- Built-in native reporters: `ai`, `console`, `consoleFull`, `csv`, `html`,
  `json`, `markdown`, `silent`, `sarif`, `threshold`, `xcode`, `xml`, and
  `badge`.
- Native `git blame -w` support in reports.
- Native Rust API for path-based detection, in-memory `SourceFile` detection,
  an embeddable argv runner, native tokenizer map generation, native
  `Detector`/`Statistic`/`MemoryStore` counterparts, upstream-style default
  options, argv option parsing, supported format listing, format lookup, and
  both `detect_clones_and_statistic` and
  `detect_clones_and_statistics` spellings.
- Source-build npm package metadata and bin shims for `npx jscpd-rs`,
  `jscpd`, and `jscpd-server`.
- Public benchmark suite on pinned React, Next.js, and Prometheus revisions.

### Compatibility And Performance

The first release is intentionally coverage-first: Rust must not miss duplicated
upstream lines on the same inputs/options. Additional Rust findings are allowed
while compatibility converges and remain visible in comparison output.

Recorded release-candidate public benchmark measurements from
`scripts/release-candidate.sh`:

| Case | Commit | Format | Rust avg | Upstream avg | Speedup | Compat |
| --- | --- | --- | ---: | ---: | ---: | --- |
| React | `f0dfee3` | JavaScript | 0.199097s | 10.079214s | 50.62x | pass |
| Next.js | `2bbb67b9` | TypeScript | 0.262433s | 14.715736s | 56.07x | pass |
| Prometheus | `a0524ee` | Go | 0.085239s | 4.642435s | 54.46x | pass |

### Known First-Release Deviations

- Dynamic npm reporters, stores, listeners, and plugins are not loaded.
- External reporter and store names keep upstream-style warning/fallback
  behavior where upstream continues.
- Exact clone pair ordering, token totals, and fragment boundaries remain
  diagnostic as long as upstream duplicated lines are covered.
- HTML output is self-contained and practically compatible, not pixel-perfect.
- The Rust crate exposes a native Rust API, not the upstream JavaScript package
  API.
- The `jscpd-rs@0.1.0` npm package builds native binaries from source during
  install.
- Full Prism grammar parity for every long-tail format is not attempted in this
  release. Formats should be promoted from generic tokenization when concrete
  coverage gates show missed upstream lines.
