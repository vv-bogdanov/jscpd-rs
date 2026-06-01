# jscpd-rs User Guide

`jscpd-rs` is a 50x+ faster duplicate-code detector for local development,
CI/CD, and code quality gates. It scans source trees, detects copy-paste
fragments across files, writes console, JSON, SARIF, HTML, XML, CSV, Markdown,
badge, and Xcode reports, fails CI on duplication thresholds, and serves
snippet checks over HTTP/MCP.

It implements the common upstream `jscpd` CLI workflow with native Rust
performance: upstream-style flags, `.jscpd.json` and `package.json#jscpd`
configuration, Git blame, report generation, exit-code behavior, and a native
server.

This guide is intentionally user-facing. Release policy, benchmark evidence,
and compatibility details live in the release docs linked from the README.

## Installation

Install the Rust binaries from crates.io:

```bash
cargo install jscpd-rs --locked
```

Install through npm/npx when Node is already part of the workflow:

```bash
npm install -g jscpd-rs
npx jscpd-rs --version
```

`jscpd-rs@0.1.2+` installs prebuilt Linux, macOS, and Windows binaries where
available and falls back to building from source with Cargo for unsupported
platforms. The original `0.1.0` npm package was source-build only. See the
[prebuilt binary distribution plan](prebuilt-binaries.md).

Install from a checkout:

```bash
git clone https://github.com/vv-bogdanov/jscpd-rs.git
cd jscpd-rs
cargo install --path . --bins --locked
```

## CLI Usage

Run a scan:

```bash
jscpd .
```

Scan selected paths and formats:

```bash
jscpd --format javascript,typescript apps packages
```

Tune clone size:

```bash
jscpd --min-lines 5 --min-tokens 50 src
```

Write machine-readable reports:

```bash
jscpd --reporters console,json,sarif --output report src
```

Fail CI when the duplicated-line percentage is too high:

```bash
jscpd --threshold 5 --exitCode 1 .
```

Inspect the upstream-compatible help and format registry:

```bash
jscpd --help
jscpd --list
```

## Common Options

| Option | Purpose | Default |
| --- | --- | --- |
| `--min-lines`, `-l` | minimum clone size in source lines | `5` |
| `--min-tokens`, `-k` | minimum clone size in tokens | `50` |
| `--max-lines`, `-x` | skip sources with more lines | `1000` |
| `--max-size`, `-z` | skip sources above a byte size such as `100kb` or `1mb` | `100kb` |
| `--threshold`, `-t` | fail when duplicated percentage is above the threshold | unset |
| `--exitCode` | exit code to use when duplication is detected | upstream default |
| `--format`, `-f` | comma-separated format allowlist | all formats |
| `--pattern`, `-p` | glob pattern used during discovery | `**/*` |
| `--ignore`, `-i` | comma-separated ignore globs | unset |
| `--ignore-pattern` | skip code blocks matching regular expressions | unset |
| `--mode`, `-m` | detection mode: `strict`, `mild`, or `weak` | `mild` |
| `--skipComments` | shorthand for `--mode weak` | `false` |
| `--reporters`, `-r` | comma-separated reporter list | `console` |
| `--output`, `-o` | report output directory | `./report` |
| `--blame`, `-b` | add Git author/date information to duplicate fragments | `false` |
| `--absolute`, `-a` | write absolute source paths in reports | `false` |
| `--noSymlinks`, `-n` | do not follow symlinks during discovery | `false` |
| `--gitignore` / `--no-gitignore` | enable or disable Git ignore handling | enabled |
| `--skipLocal` | report only cross-folder duplication | `false` |
| `--noTips` | suppress post-run tips and promotional output | `false` |

The CLI accepts multiple `--ignore` flags and comma-separated ignore lists:

```bash
jscpd --ignore "target/**" --ignore "node_modules/**,dist/**" .
```

## Configuration

`jscpd-rs` reads `.jscpd.json` by default when present. It also reads a `jscpd`
section from `package.json`, matching the common upstream workflow.

Example `.jscpd.json`:

```json
{
  "path": ["src", "packages"],
  "format": ["javascript", "typescript", "rust"],
  "minLines": 5,
  "minTokens": 50,
  "threshold": 5,
  "reporters": ["console", "json", "html", "sarif"],
  "output": "report",
  "ignore": ["target/**", "node_modules/**", "dist/**"],
  "gitignore": true,
  "noTips": true
}
```

Example `package.json` section:

```json
{
  "jscpd": {
    "path": ["src"],
    "reporters": ["console", "json"],
    "threshold": 5,
    "ignore": ["coverage/**", "**/*.snap"]
  }
}
```

CLI arguments are applied after config loading, so command-line values can
override project defaults.

## Reporters

Built-in native reporters:

| Reporter | Output |
| --- | --- |
| `console` | compact terminal summary and clone list |
| `consoleFull` | terminal output with duplicated fragments |
| `ai` | compact, token-efficient clone list for agent prompts |
| `json` | `jscpd-report.json` |
| `xml` | `jscpd-report.xml` |
| `csv` | `jscpd-report.csv` |
| `markdown` | `jscpd-report.md` |
| `html` | self-contained HTML report under `html/` |
| `sarif` | `jscpd-sarif.json` for code-scanning pipelines |
| `xcode` | diagnostics formatted for Xcode-style tooling |
| `threshold` | threshold-only error output |
| `badge` | `jscpd-badge.svg` |
| `silent` | suppress report output |

Dynamic npm reporters are intentionally not loaded in the first release. Unknown
external reporter names keep upstream-style warnings and continue where upstream
continues.

## Server And MCP

Start the native server:

```bash
jscpd-server . --host 127.0.0.1 --port 3000
```

Health and statistics:

```bash
curl http://127.0.0.1:3000/api/health
curl http://127.0.0.1:3000/api/stats
```

Check a snippet against the scanned project:

```bash
curl -X POST http://127.0.0.1:3000/api/check \
  -H "Content-Type: application/json" \
  -d '{"format":"javascript","code":"function sum(a,b){return a+b;}"}'
```

Refresh the project token maps after changing files:

```bash
curl -X POST http://127.0.0.1:3000/api/recheck
```

MCP endpoint:

```text
http://127.0.0.1:3000/mcp
```

Example MCP client entry:

```json
{
  "mcpServers": {
    "jscpd": {
      "type": "streamable-http",
      "url": "http://127.0.0.1:3000/mcp"
    }
  }
}
```

The server exposes core duplication tools and a statistics resource over native
JSON-RPC HTTP.

## Rust API

Path-based detection:

```rust
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut options = jscpd_rs::get_default_options();
    options.paths = vec![PathBuf::from("src")];
    options.reporters.clear();
    options.silent = true;

    let result = jscpd_rs::detect_clones_and_statistics(&options)?;
    println!("{} clones", result.clones.len());
    Ok(())
}
```

In-memory detection:

```rust
let mut options = jscpd_rs::get_default_options();
options.reporters.clear();
options.min_lines = 2;
options.min_tokens = 5;

let files = vec![
    jscpd_rs::SourceFile {
        source_id: "a.js".to_string(),
        format: "javascript".to_string(),
        content: "const a = 1;\nconst b = 2;\nconst c = a + b;\n".to_string(),
    },
    jscpd_rs::SourceFile {
        source_id: "b.js".to_string(),
        format: "javascript".to_string(),
        content: "const a = 1;\nconst b = 2;\nconst c = a + b;\n".to_string(),
    },
];

let result = jscpd_rs::detect_source_files(files, &options);
assert!(!result.clones.is_empty());
```

Useful entry points:

- `get_default_options`
- `get_options_from_args`
- `detect_clones`
- `detect_clones_and_statistics`
- `detect_source_files`
- `get_supported_formats`
- `get_format_by_file`
- `Tokenizer`
- `Detector`
- `MemoryStore`

## AI Skills

Install the tool-reference and dry-refactoring skills:

```bash
npx skills add vv-bogdanov/jscpd-rs --skill jscpd
npx skills add vv-bogdanov/jscpd-rs --skill dry-refactoring
```

The `ai` reporter is designed for short clone summaries that can be passed to
coding agents without including duplicated source fragments by default:

```bash
jscpd --reporters ai src
```

## Compatibility Notes

The first release target is practical, coverage-first upstream compatibility.
For the same inputs and options, `jscpd-rs` must not miss duplicated source
lines reported by upstream `jscpd`. Extra Rust findings are allowed while
compatibility converges and remain visible in compatibility reports.

Intentional first-release limits:

- dynamic npm reporters, stores, listeners, and plugins are not loaded;
- exact token totals and pair ordering may differ from upstream while
  duplicated upstream lines remain covered;
- HTML output is self-contained and practically compatible, not pixel-perfect;
- the Rust crate exposes a native Rust API, not the upstream JavaScript API.

See `docs/release-decisions.md` and `docs/compat-baseline.md` for the current
release policy and compatibility evidence.
