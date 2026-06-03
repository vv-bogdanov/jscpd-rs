# jscpd-rs

[![crates.io](https://img.shields.io/crates/v/jscpd-rs.svg?style=flat-square)](https://crates.io/crates/jscpd-rs) [![npm](https://img.shields.io/npm/v/jscpd-rs.svg?style=flat-square)](https://www.npmjs.com/package/jscpd-rs) [![license](https://img.shields.io/github/license/vv-bogdanov/jscpd-rs.svg?style=flat-square)](LICENSE) [![rust](https://img.shields.io/badge/rust-1.93%2B-dea584?style=flat-square)](https://www.rust-lang.org/)

[![release-gate](https://github.com/vv-bogdanov/jscpd-rs/actions/workflows/release-gate.yml/badge.svg)](https://github.com/vv-bogdanov/jscpd-rs/actions/workflows/release-gate.yml) [![docs.rs](https://img.shields.io/docsrs/jscpd-rs?style=flat-square)](https://docs.rs/jscpd-rs) [![Socket.dev](https://badge.socket.dev/npm/package/jscpd-rs)](https://socket.dev/npm/package/jscpd-rs) [![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/vv-bogdanov/jscpd-rs/badge)](https://scorecard.dev/viewer/?uri=github.com/vv-bogdanov/jscpd-rs)

50x+ faster duplicate-code detector for local development, CI/CD, and code
quality gates.
`jscpd-rs` scans a codebase, finds copy-paste fragments across files, writes
console, JSON, SARIF, HTML, XML, CSV, Markdown, badge, and Xcode reports, and
can fail a build when duplication crosses a configured threshold.

It is a native Rust implementation of the common
[`jscpd`](https://github.com/kucherenko/jscpd) command-line workflow:
upstream-style CLI flags, `.jscpd.json` and `package.json#jscpd`
configuration, report formats, exit-code behavior, Git blame, and server
snippet checks. The practical goal is simple: keep copy-paste detection
always-on without spending unnecessary CI minutes, developer waiting time,
cloud compute budget, or electricity on repeated quality gates.

Recorded public benchmark baselines show 50x+ speedups over upstream on the
covered repositories. The compatibility gate is coverage-first: on the same
inputs and options, `jscpd-rs` must not miss duplicated source lines reported
by upstream `jscpd`.

## Install

Cargo:

```bash
cargo install jscpd-rs --locked
jscpd --version
```

npm/npx:

```bash
npm install -g jscpd-rs
jscpd --version

npx jscpd-rs --version
npx jscpd-rs .
```

Current npm packaging note: `jscpd-rs` installs prebuilt Linux, macOS, and
Windows binaries through optional platform packages and does not run
install-time build scripts. Unsupported npm platforms should use Cargo. See
[docs/prebuilt-binaries.md](docs/prebuilt-binaries.md).

From this repository:

```bash
git clone https://github.com/vv-bogdanov/jscpd-rs.git
cd jscpd-rs
cargo install --path . --bins --locked
```

Build without installing:

```bash
cargo build --release --bin jscpd
cargo build --release --bin jscpd-server
```

## Quick Start

Scan a project:

```bash
jscpd .
```

Tune the detection threshold:

```bash
jscpd --min-lines 5 --min-tokens 50 src
```

Generate reports:

```bash
jscpd --reporters console,json,html --output report src
```

Fail CI when duplication is above a threshold:

```bash
jscpd --threshold 5 --exitCode 1 src
```

Use the upstream-compatible command help and format list:

```bash
jscpd --help
jscpd --list
```

Start the native REST/MCP server:

```bash
jscpd-server . --host 127.0.0.1 --port 3000
curl http://127.0.0.1:3000/api/health
```

The current server exposes `/`, `/api/health`, `/api/stats`, `/api/check`,
`/api/recheck`, and `/mcp`. Snippet checks reuse project token maps refreshed
by `/api/recheck`.

For full CLI, configuration, reporter, server, MCP, and Rust API examples, see
[docs/user-guide.md](docs/user-guide.md).
If you already use upstream `jscpd`, see
[docs/migrating-from-jscpd.md](docs/migrating-from-jscpd.md).

## GitHub Actions

Install from crates.io:

```yaml
jobs:
  duplication:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install jscpd-rs --locked
      - run: jscpd src --reporters console,json --threshold 5 --exitCode 1
```

Use npm/npx when a Node-based CI environment is already available:

```yaml
jobs:
  duplication:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-node@v5
        with:
          node-version: 22
      - run: npx jscpd-rs src --reporters console,json --threshold 5 --exitCode 1
```

Install from a checked-out source tree:

```yaml
jobs:
  duplication:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install --path . --bins --locked
      - run: jscpd src --reporters console,json --threshold 5 --exitCode 1
```

## AI Refactoring Skills

Install the project skills for duplication detection and guided dry
refactoring:

```bash
npx skills add vv-bogdanov/jscpd-rs --skill jscpd
npx skills add vv-bogdanov/jscpd-rs --skill dry-refactoring
```

## Why jscpd-rs

- **Fast CI/CD gates:** duplicate detection should be cheap enough to run on
  every pull request.
- **Low-friction rollout:** npm installs use prebuilt binaries on supported
  Linux, macOS, and Windows targets without install-time build scripts. Other
  platforms should install through Cargo.
- **Actionable reports:** console, JSON, SARIF, HTML, XML, CSV, Markdown,
  badge, Xcode, threshold, and AI-oriented reports are implemented natively.
- **Lower operating cost:** shorter scans reduce paid compute minutes and
  repeated developer wait time.
- **Native detector path:** the detector does not embed or spawn JavaScript for
  core behavior.
- **Practical compatibility:** the CLI, config, reporter, server, and exit-code
  workflows are designed to map onto common upstream `jscpd` usage.
- **Small project-specific core:** use battle-tested Rust crates for CLI
  parsing, config formats, ignore handling, serialization, token processing,
  concurrency, and reporting wherever that keeps the implementation simpler.

## What Works Today

The current release is a coverage-first compatible CLI replacement for common
`jscpd` duplicate-code and copy-paste detection workflows:

- `jscpd` and `jscpd-server` binaries with upstream-compatible command names;
- CLI and config option surface covered by compatibility scripts;
- native built-in reporters: `ai`, `console`, `consoleFull`, `csv`, `html`,
  `json`, `markdown`, `silent`, `sarif`, `threshold`, `xcode`, `xml`, and
  `badge`;
- upstream-synchronized format registry with native JS/TS/JSX/TSX tokenization
  and generic native tokenization for long-tail formats;
- native Git blame support;
- native REST/MCP server workflow;
- Rust library API for running detection from paths or prepared in-memory
  sources.

Dynamic npm reporters, stores, listeners, and plugins are intentionally out of
scope for the first release. Unknown external reporters/stores keep
upstream-style warnings and continue where upstream continues.

## Compatibility Contract

The release gate is coverage-first. For the same inputs and options,
`jscpd-rs` must not miss duplicated lines reported by upstream `jscpd`. Extra
Rust duplicates are allowed while compatibility converges, but compatibility
reports keep them visible as `extra` findings.

Exact pair ordering and token totals are quality metrics rather than the
default blocking gate. This matters for multi-way clones: different pair
selection can still cover the same duplicated source lines.

The upstream repository is checked out as `jscpd/` and treated as the
executable specification. Compatibility scripts run both implementations and
compare their reports.

## Performance

Latest recorded public benchmark baseline for duplicate-code detection:

| Repo | Format | Rust avg | Upstream avg | Speedup |
| --- | --- | ---: | ---: | ---: |
| React | JavaScript | 0.193443s | 9.979393s | 51.59x |
| Next.js | TypeScript | 0.281938s | 14.182806s | 50.30x |
| Prometheus | Go | 0.086096s | 4.608737s | 53.53x |

Reproduce the public benchmark and coverage suite:

```bash
scripts/release-candidate.sh
```

Benchmark native server snippet checks against upstream:

```bash
RUNS=20 scripts/bench-server.sh
```

Release-candidate workflows rerun the public suite before each new publication
and enforce the core coverage gate, so README numbers stay tied to a concrete
commit and gate output. The public release gate fails below a 45x speedup on
the default benchmark cases to prevent silent performance regressions while
preserving room for normal runner noise.

## Library API

The crate exposes the detector core for native integrations:

```rust
let options = jscpd_rs::get_default_options();
let result = jscpd_rs::detect_clones_and_statistic(&options)?;
let clones = result.clones;

let clones = jscpd_rs::jscpd(["jscpd", "src", "--silent", "--noTips"])?;
```

`detect_clones_and_statistics` is also available as the idiomatic Rust spelling.
`jscpd` and `jscpd_with_exit_callback` provide a native embeddable argv runner
similar to upstream `jscpd(argv, exitCallback?)`. `get_options_from_args` parses
upstream-style argv into normalized `Options` for native integrations.

`Tokenizer` provides a native generate-maps entrypoint over the same tokenizer
used by detection. `Detector`, `Statistic`, and `MemoryStore` expose native
counterparts for the main upstream core classes without loading JavaScript.
`detect_source_files` accepts in-memory `SourceFile` values, which is the
foundation for the upstream-style snippet/server workflow. Format helpers are
available through `get_supported_formats`, `get_format_by_file`, and
`get_format_by_file_with_mappings`.

## Architecture

The implementation keeps the hot path small and native:

```text
paths/config
  -> ignore-aware discovery
  -> native token maps
  -> parallel duplicate detection
  -> reporters / exit codes / server API
```

Core crates and libraries:

- `clap`, `serde`, and config parsers for the CLI/config surface;
- `ignore`, `globset`, and Git ignore handling for file discovery;
- Oxc-backed JS/TS/JSX/TSX token processing for the highest-volume languages;
- native generic tokenizers for long-tail formats and embedded code blocks;
- `rayon` and Rust data structures for parallel discovery/detection work;
- native reporters for JSON, SARIF, XML, CSV, Markdown, HTML, console, badge,
  Xcode, threshold, silent, and AI refactoring output.

## Known First-Release Deviations

The first release is native-only and coverage-first. These differences from the
JavaScript package are intentional unless a real workflow proves otherwise:

- dynamic npm reporters, stores, listeners, and plugins are not loaded;
- token totals and exact clone pair ordering may differ from Prism-based
  upstream reports while duplicated upstream lines remain covered;
- HTML reports are self-contained and practically compatible, not pixel-perfect;
- the Rust crate exposes a native Rust API, not the upstream JavaScript package
  API.

## Development

The upstream repository is checked out as the `jscpd/` git submodule and is the
executable specification for behavior.

```bash
git submodule update --init --recursive
cargo test
```

Useful focused checks:

```bash
scripts/compat-cli.sh
scripts/compat-config.sh
scripts/compat-reporters.sh
STRICT=coverage scripts/compat-matrix.sh
```

Rust code coverage is optional and intentionally kept out of the default fast
gate:

```bash
cargo install cargo-llvm-cov --locked
SUMMARY=1 scripts/coverage.sh
SCOPE=core SUMMARY=1 scripts/coverage.sh
SCOPE=core FAIL_UNDER_LINES=93 scripts/coverage.sh
```

Black-box behavior tests that exercise the public API live in `tests/`. Small
private-helper tests stay next to the module they protect.

Known upstream bug candidates and intentional compatibility exceptions are
tracked in [docs/upstream-bugs.md](docs/upstream-bugs.md). GitHub-ready issue
drafts are prepared in
[docs/upstream-issue-drafts.md](docs/upstream-issue-drafts.md).

## Release Gates

Fast local gate:

```bash
scripts/release-gate.sh
```

The fast gate includes `cargo fmt`, `cargo test`, shell syntax checks,
`shellcheck`, package/install checks, and the focused compatibility gates.

Package/install gate:

```bash
scripts/package-check.sh
```

npm package/npx gate:

```bash
scripts/npm-package-check.sh
```

Full compatibility matrix:

```bash
FULL=1 scripts/release-gate.sh
```

Release candidate gate:

```bash
scripts/release-candidate.sh
```

The GitHub Actions workflow runs the fast gate on pushes and pull requests.
Manual workflow runs can enable the full compatibility matrix and public
benchmark suite before a release, or set `release_candidate=true` to run the
full release-candidate gate in CI.

See [docs/compat-baseline.md](docs/compat-baseline.md) for the current gate
baseline, [docs/release-readiness.md](docs/release-readiness.md) for component
status, [docs/release-checklist.md](docs/release-checklist.md) for the
publication checklist, [CHANGELOG.md](CHANGELOG.md) for release notes, and
[docs/release-decisions.md](docs/release-decisions.md) for approved
first-release compatibility decisions.

## License

MIT
