# Release Readiness

Last updated: 2026-06-01.

This is the working component checklist for the first release. The authoritative
policy decisions are still in `docs/release-decisions.md`; this file tracks the
current implementation status.

## Ready For First Release

| Component | Status | Notes |
| --- | --- | --- |
| Binary/package surface | ready | `jscpd` binary name, Cargo package include list, publish dry-run, install check, version/help smoke checks, npm package metadata, and local `npx` smoke checks. |
| CLI option surface | ready | Main upstream flags are parsed, including visible Commander quirks gated by `scripts/compat-cli.sh`. |
| Config loading | ready | `.jscpd.json` and `package.json#jscpd`, config-relative paths/ignore, malformed JSON behavior, symlinked explicit config paths. |
| File discovery | ready | Format filters, custom extensions/names, `.gitignore`, global Git excludes, symlink policy, shebang detection, max size/line filtering. |
| Format registry | ready | Generated from upstream tokenizer build; current registry has 223 formats and 206 extension mappings. |
| Detector core | ready | Numeric hashes, compact token streams, per-format sharding, parallel preparation/detection, coverage-first comparator. |
| Hot JS/TS tokenization | ready | Native Oxc-backed paths for JavaScript, TypeScript, JSX, and TSX with coverage gates. |
| Embedded/block formats | ready | Markdown, markup, Vue, Svelte, Astro, Apex, and TAP have native block handling where needed for upstream coverage. |
| Built-in reporters | ready | `ai`, `console`, `consoleFull`, `csv`, `html`, `json`, `markdown`, `silent`, `sarif`, `threshold`, `xcode`, `xml`, and `badge`; file reporters are gated for clone and no-duplicate reports. |
| Blame | ready | Native `git blame -w` data is populated and gated by `scripts/compat-blame.sh`. |
| Native Rust API | ready | `jscpd`, `jscpd_with_exit_callback`, `Tokenizer`, `Detector`, `Statistic`, `MemoryStore`, `detect_clones`, `detect_clones_and_statistic`, `detect_clones_and_statistics`, `detect_source_files`, default options, argv option parsing, supported formats, and format lookup helpers expose the app, tokenizer, detector, statistics, and store core for path-based and in-memory integrations. See `docs/api-parity.md`. |
| Native server | partial | `jscpd-server` exposes `/`, `/api/health`, `/api/stats`, `/api/check`, `/api/recheck`, and `/mcp`; exact help text, stable CLI, HTTP success/error, and MCP contracts are gated; `/api/check` reuses prepared project token maps; exact upstream Streamable HTTP SDK behavior remains follow-up. |
| Performance harness | ready | Local benchmark script and public benchmark suite with pinned output recording and speedup gates. |
| Server benchmark harness | ready | `scripts/bench-server.sh` compares native and upstream `/api/check` latency on the same initialized project and snippet payload. Keep it advisory until real server usage needs a blocking gate. |
| Release gates | ready | Default CI gate, full compatibility matrix, Cargo/npm package checks, reporter/config/CLI/blame gates. The default gate prints per-step timings, caches Cargo/pnpm/upstream build artifacts, and uses target-reuse npm package smoke in push/PR CI while keeping cold npm source-build in release-candidate/prepublish gates. GitHub npm/crates publish workflows now run `scripts/release-candidate.sh` before publishing. |
| Code coverage tooling | ready | One script: `scripts/coverage.sh`. Use `SCOPE=full` for the full advisory report and `SCOPE=core` for core coverage that runs all test targets while excluding CLI/server glue from the report. Local baseline on 2026-06-01: full 91.54% line / 90.13% region coverage; core 93.18% line / 91.39% region coverage. `scripts/release-candidate.sh` enforces `SCOPE=core FAIL_UNDER_LINES=93 scripts/coverage.sh` by default. |

## Partial Or Follow-Up

| Component | Status | Recommended action |
| --- | --- | --- |
| Long-tail tokenization | coverage-first | Keep generic tokenization by default. Promote formats only when fixtures or public repos show missed upstream coverage. |
| Exact pair parity | diagnostic | Do not block release while every upstream duplicated line is covered. Reduce noisy extras after user-facing reports become annoying. |
| Token totals | diagnostic | Native token streams may differ from Prism. Keep report-visible clone coverage as the gate. |
| HTML reporter polish | practical parity | Keep self-contained HTML stable. Do not chase pixel-perfect upstream parity for the first release. |
| Terminal cosmetics | practical parity | Important messages are gated; exact wrapping/order remains lower priority. |
| Upstream JavaScript API parity | follow-up | Native Rust helpers cover the practical app/tokenizer/detector/statistics/store concepts, including an embeddable argv runner and tokenizer map generation; exact JS package export shape is not implemented in the Rust crate. See `docs/api-parity.md`. |
| Server snippet matching | optimized baseline | Native `/api/check` and MCP `check_duplication` are functional and reuse project token maps from the last scan; use `scripts/bench-server.sh` before adding a dedicated window index. |
| Npm prebuilt binaries | ready | Platform-specific optional package metadata, runtime prebuilt resolution, prebuilt/fallback package checks, and GitHub Release publishing automation are wired for `jscpd-rs@0.1.2+`. The main npm package is blocked if any configured platform package fails or is missing. See the [prebuilt binary distribution plan](prebuilt-binaries.md). |
| Latest full publication gate | ready | `scripts/prepublish-check.sh` passed locally on code commit `8c3da0e`, including `scripts/release-candidate.sh`, package/install verification, crate/tag availability checks, npm package/name/npx verification, and `cargo publish --dry-run --locked`. GitHub Actions default `release-gate` passed on code commit `8c3da0e` in run `26710762680`. After benchmark documentation updates, `RUN_RELEASE_CANDIDATE=0 scripts/prepublish-check.sh` is the package/dry-run refresh gate for the exact package contents being tagged. |

## Post-MVP

| Component | Decision |
| --- | --- |
| Dynamic npm reporters | Do not implement for the first release; keep upstream-style missing-package warnings. |
| Dynamic npm stores | Do not implement for the first release; default in-memory store is the release path. |
| Listeners/plugins runtime | Option-surface compatibility only unless a real workflow requires native support. |
| MCP endpoint polish | Core native endpoint exists; tighten exact SDK edge cases only when MCP client compatibility demands it. |
| Persistent cache/store backends | Add only if public benchmark data proves the in-memory path is insufficient. |
| Full Prism grammar port | Do not rewrite all grammars eagerly; use native crates or small scanners only for proven gaps. |
| CI coverage enforcement | Keep coverage out of the default fast gate until CI runtime is measured; the core coverage gate now runs in release-candidate flows. |
