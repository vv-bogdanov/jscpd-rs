# Release Checklist

This checklist is the publication runbook for Rust releases.
Policy decisions live in `docs/release-decisions.md`; current evidence lives in
`docs/compat-baseline.md` and `docs/release-readiness.md`.

## Current Release Candidate Evidence

Latest full local prepublish gate:

```bash
scripts/prepublish-check.sh
```

Run on the exact release-candidate checkout before tagging. This includes
`scripts/release-candidate.sh`, package/install verification, crate/tag
availability checks, npm package/name/npx verification, and
`cargo publish --dry-run --locked`. Documentation-only commits may reuse fresh
release-candidate evidence only if they do not change code, scripts, package
metadata, or benchmark configuration; rerun
`RUN_RELEASE_CANDIDATE=0 scripts/prepublish-check.sh` after documentation edits
so package/dry-run evidence matches the exact package contents being tagged.

GitHub Actions default `release-gate` must pass on the exact pushed commit
being published. Check the current run in GitHub Actions after the final push;
the publish blocker below is the authoritative gate.

Latest GitHub Actions default release-gate:

```text
push
```

Latest known-good release-publish workflow before this checklist refresh:
GitHub Release `v0.1.9` passed on 2026-06-03 at commit `68eb0ba`:
https://github.com/vv-bogdanov/jscpd-rs/actions/runs/26866531012

CI timing snapshot after the first cache/timing pass, from cold GitHub Actions
run `26710415211` on commit `7bdf12f`:

| Step | Time |
| --- | ---: |
| `package/install check` | 192s |
| `CLI compatibility` | 56s |
| `cargo test` | 35s |
| `server compatibility` | 28s |
| `config compatibility` | 4s |

Optimized default CI run `26710686373` on commit `71088f1` completed in
2m18s. Current warm-cache bottlenecks:

| Step | Time |
| --- | ---: |
| `CLI compatibility` | 36s |
| `server compatibility` | 29s |
| `package/install check` | 23s |
| `cargo test` | 6s |
| `config compatibility` | 6s |

The workflow now restores Cargo, pnpm, and upstream build caches, resolves the
pnpm store with upstream's `pnpm@10.28.0`, validates restored pnpm
`node_modules` symlinks before using upstream builds, skips `clippy`
installation outside release-candidate runs, and uses a local prebuilt npm
package smoke without install-time Cargo builds.

Recorded public benchmark baseline for the current release evidence:

| Case | Commit | Format | Rust avg | Upstream avg | Speedup | Compat |
| --- | --- | --- | ---: | ---: | ---: | --- |
| `react` | `f0dfee3` | `javascript` | 0.197325s | 10.413453s | 52.77x | pass |
| `next` | `2bbb67b9` | `typescript` | 0.270786s | 14.983243s | 55.33x | pass |
| `prometheus` | `a0524ee` | `go` | 0.083162s | 4.842499s | 58.23x | pass |

## Publish Blockers

Before publishing, all of these must be true:

- `git status --short` is clean.
- `git submodule status jscpd` points at the reviewed upstream reference.
- `scripts/release-candidate.sh` passes on the exact code commit being tagged.
- GitHub Actions `release-gate` passes on the pushed commit.
- GitHub Actions `release-publish` runs one release-candidate preflight before
  publishing crates.io and npm packages. Manual `crates-publish` and
  `npm-publish` reruns keep `run_release_candidate=true` unless the exact same
  commit already has fresh release-candidate evidence.
- `scripts/package-check.sh` passes and the package file list excludes
  `jscpd/`, `target/`, `node_modules/`, and `scripts/`.
- `scripts/npm-package-check.sh` passes, including `npm pack`,
  `npm publish --dry-run --json`, local install smoke checks for `jscpd-rs`,
  `jscpd`, and `jscpd-server`, and an `npx --package <tarball>` smoke run.
- `scripts/cargo-deny-check.sh` passes, covering Rust advisories, yanked
  crates, license allowlist, and source registry policy.
- `scripts/actionlint.sh` passes for GitHub Actions workflow syntax.
- `cargo rustdoc --lib -- -D missing_docs` passes, so docs.rs public API
  documentation cannot silently regress.
- `cargo publish --dry-run --locked` passes for the exact package manifest and
  include list being published.
- `README.md`, `docs/compat-baseline.md`, and
  `docs/public-benchmark-suite.md` contain the same recorded public benchmark
  numbers.
- After npm publication, `node scripts/socket-package-check.mjs` passes against
  the main npm package and every optional prebuilt platform package. The main
  package must keep 100% Supply Chain Security and 100% Maintenance on Socket;
  optional native platform packages use lower explicit thresholds because Socket
  can score binary-only packages differently.
- After npm publication, `NPM_REGISTRY_REQUIRE_PUBLISHED=1
  scripts/npm-registry-check.sh` passes, verifying registry integrity,
  signatures, SLSA provenance attestations, and `npm audit signatures`.
- For a new publication, the target crate version, npm package versions, and
  `vX.Y.Z` Git tag are not already published, unless an explicitly documented
  target-only npm rerun is being used for a prebuilt package.
- For npm publication, every configured prebuilt optional package is published
  before the main `jscpd-rs` package; target-only reruns use
  `publish_main=false`.
- `docs/upstream-bugs.md` contains concrete repro commands for upstream issues
  we plan to file.
- `docs/upstream-issue-drafts.md` contains reviewed issue drafts ready to
  verify against current upstream and post.
- `CHANGELOG.md` contains the exact release notes for the version being tagged.

## First-Release Scope

Treat these as release scope:

- `jscpd` and `jscpd-server` binaries with upstream-compatible command names.
- CLI/config option surface covered by `scripts/compat-cli.sh` and
  `scripts/compat-config.sh`.
- Coverage-first duplicate parity: Rust must not miss duplicated upstream lines
  for the same inputs and options.
- Built-in native reporters: `ai`, `console`, `consoleFull`, `csv`, `html`,
  `json`, `markdown`, `silent`, `sarif`, `threshold`, `xcode`, `xml`, and
  `badge`.
- Native file discovery, format registry, JS/TS/JSX/TSX tokenization, generic
  long-tail tokenization, blame, native API, and native server endpoints listed
  in `docs/release-readiness.md`.

## Intentional 0.x Deviations

These are not publication blockers for the current 0.x line:

- Dynamic npm reporters, stores, listeners, and plugins are not loaded. The
  compatible option surface and upstream-style missing-package warnings are the
  release contract.
- Exact token totals, pair ordering, and boundaries are diagnostic while
  upstream duplicated lines remain covered.
- HTML output is self-contained and practically compatible, not pixel-perfect.
- The Rust crate exposes a native Rust API, not the upstream JavaScript package
  API.
- Persistent store/cache backends remain demand-driven until benchmark data
  shows the in-memory path is insufficient.
- Full Prism grammar parity for every long-tail format is not attempted; promote
  formats only when coverage gates show missed upstream lines.

## Pre-Tag Commands

Run from the repository root:

```bash
scripts/prepublish-check.sh
```

The script checks clean git state, the reviewed `jscpd` submodule reference,
local and remote tag availability, exact crate version availability through
`cargo info`, exact npm package version availability through `npm view`,
benchmark-number consistency across release docs, the full release-candidate
gate, package/install validation, npm pack/npx validation, and
`cargo publish --dry-run --locked`. The npm availability check covers the main
package and every prebuilt optional package. Set
`RUN_RELEASE_CANDIDATE=0` only when the same code commit already has fresh
local and CI release-candidate evidence.

Then push the exact release commit and verify the GitHub Actions
`release-gate` result. Use the workflow dispatch `release_candidate` input for a
full CI-side release-candidate run when needed.

Historical first publication candidate checked on 2026-05-31: local and remote
`v0.1.0` tag lookups returned no entries. `cargo search jscpd-rs --limit 5`
returned no exact crate, `npm view jscpd-rs version` returned `E404`, and the
sparse crates.io index path
`https://index.crates.io/js/cp/jscpd-rs` returned 404.

## Post-Tag Smoke

After tagging or publishing, install the package into a temporary Cargo root and
check the binaries:

```bash
cargo install --path . --bin jscpd --root /tmp/jscpd-rs-install --force --locked
cargo install --path . --bin jscpd-server --root /tmp/jscpd-rs-install --force --locked
/tmp/jscpd-rs-install/bin/jscpd --version
/tmp/jscpd-rs-install/bin/jscpd --help
/tmp/jscpd-rs-install/bin/jscpd-server --version
```

The package gate itself installs both binaries with `cargo install --bins`; the
two explicit commands above are a manual smoke equivalent for post-tag checks.

## Next Release Themes

Track these after the current release:

- Reduce noisy extra Rust findings where they are user-visible false positives.
- Monitor npm prebuilt install behavior and add Linux musl or Windows arm64
  packages only when install data or user reports show demand; see the
  [prebuilt binary distribution plan](prebuilt-binaries.md).
- Add native persistent store/cache only if release-scale benchmark data needs
  it.
- Tighten MCP Streamable HTTP SDK edge cases if real MCP clients require them.
- Promote long-tail tokenizers only from concrete missed-coverage evidence.
- File upstream bug reports from `docs/upstream-issue-drafts.md`.

## Automated GitHub Release Publishing

After the first manual Cargo/npm bootstrap, future releases should use GitHub
Release publication as the single publish trigger.

Configured workflows:

- `.github/workflows/release-publish.yml`
- `.github/workflows/release-candidate.yml`
- `.github/workflows/crates-publish.yml`
- `.github/workflows/npm-publish.yml`

`release-publish` runs on `release.published` for non-draft, non-prerelease
GitHub Releases. It calls `release-candidate` once, then calls `crates-publish`
and `npm-publish` with `run_release_candidate=false` so the expensive gate is
not duplicated.

`release-candidate` splits expensive release checks into parallel jobs:

- one core release gate without the full compatibility matrix or public
  benchmarks,
- four compatibility-matrix shards,
- one public-benchmark job per benchmark case.

`crates-publish` and `npm-publish` are reusable/manual workflows. Both check out
the requested release tag and verify that `vX.Y.Z` matches the package version
before publishing. `crates-publish` publishes the Rust crate to crates.io;
docs.rs builds documentation automatically after crates.io accepts the crate.
`npm-publish` builds and publishes prebuilt npm platform packages before
publishing the main npm package.

Release flow for future versions:

1. Update `Cargo.toml` and `package.json` to the same version.
2. Update `package.json#optionalDependencies` to the same version for every
   platform package.
3. Update `CHANGELOG.md`, README benchmark numbers, and release docs.
4. Run the release gates, including `scripts/package-check.sh`.
5. Commit and push `main`.
6. Create and publish a GitHub Release with tag `vX.Y.Z`.
7. Confirm `release-publish` completed its `release-candidate` job and both
   publish jobs successfully.
8. Confirm `npm-publish` built and published all platform packages before the
   main `jscpd-rs` package.
9. Check crates.io, docs.rs, and npm package pages.

Trusted Publishing setup:

- crates.io: configure a trusted publisher for repository
  `vv-bogdanov/jscpd-rs` and workflow `crates-publish.yml`; keep
  `CARGO_REGISTRY_TOKEN` as the fallback until one full release succeeds
  without it.
- npm: configure a trusted publisher for repository `vv-bogdanov/jscpd-rs` and
  workflow `release-publish.yml` on `jscpd-rs` and every platform package
  listed in `docs/prebuilt-binaries.md`. npm validates the calling workflow when
  `workflow_call` is used, so manual `npm-publish.yml` reruns should keep using
  the token fallback.

Use no GitHub environment name unless the workflow is updated to declare one.
After Trusted Publishing works, revoke temporary npm/crates tokens and keep
token-based publishing as an emergency fallback only.

## CI Speed Plan

Keep this as part of the release plan:

- Treat `package/install check`, `CLI compatibility`, `cargo test`, and
  `server compatibility` as the CI bottleneck watchlist.
- Compare future warm-cache GitHub Actions runs with optimized run
  `26710686373` and cold run `26710415211`.
- Keep default push/PR CI as a fast confidence gate; keep `clippy`, full
  matrix, and public benchmark coverage in `scripts/prepublish-check.sh` and
  the sharded `release-candidate` workflow.
- Keep future release publishing on `release-publish.yml` so crates.io and npm
  reuse the same release-candidate result instead of running duplicate full
  gates.
- Keep the pinned `cargo-llvm-cov` binary cache warm; on a cache hit the
  release-candidate setup avoids recompiling the coverage tool.
- If default CI remains above roughly three minutes after warm caches, split the
  package surface smoke from the compatibility gates into separate jobs.
