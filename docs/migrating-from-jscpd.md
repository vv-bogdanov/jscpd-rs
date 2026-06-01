# Migrating From jscpd

`jscpd-rs` is a 50x+ faster duplicate-code detector and native Rust
implementation of the common upstream `jscpd` workflow. It scans source trees,
reports copy-paste fragments across files, writes console, JSON, SARIF, HTML,
XML, CSV, Markdown, badge, and Xcode reports, and can fail CI when duplication
crosses a configured threshold.

The migration goal for the first release is practical CLI/reporting
compatibility for CI and local scans, not exact JavaScript package API parity.

## Quick Command Mapping

Cargo install:

```bash
cargo install jscpd-rs --locked
jscpd --threshold 5 --exitCode 1 src
```

npm/npx:

```bash
npx jscpd-rs --threshold 5 --exitCode 1 src
```

Common upstream command:

```bash
npx jscpd src --reporters console,json --threshold 5 --exitCode 1
```

Equivalent `jscpd-rs` command:

```bash
jscpd src --reporters console,json --threshold 5 --exitCode 1
```

When installed from npm, the package exposes `jscpd-rs`, `jscpd`, and
`jscpd-server` bin names. The `jscpd` bin is an installed alias for the native
CLI so existing scripts can be tested with minimal command changes in a
controlled environment.

## Config Files

`jscpd-rs` reads the same common config locations:

- `.jscpd.json`
- `package.json#jscpd`

Example:

```json
{
  "path": ["src", "packages"],
  "format": ["javascript", "typescript", "rust"],
  "minLines": 5,
  "minTokens": 50,
  "threshold": 5,
  "reporters": ["console", "json", "sarif"],
  "output": "report",
  "ignore": ["target/**", "node_modules/**", "dist/**"],
  "gitignore": true,
  "noTips": true
}
```

CLI arguments are applied after config loading, so command-line values can
override project defaults.

## Commonly Compatible Workflows

The first release targets these upstream-style workflows:

- local duplicate scan with `jscpd .`;
- threshold-based CI failure with `--threshold` and `--exitCode`;
- format filtering with `--format`;
- `.gitignore` and explicit `--ignore` handling;
- report generation with built-in reporters;
- Git blame reports with `--blame`;
- server startup with `jscpd-server`;
- snippet checks through the native REST/MCP server.

Built-in native reporters:

- `ai`
- `badge`
- `console`
- `consoleFull`
- `csv`
- `html`
- `json`
- `markdown`
- `sarif`
- `silent`
- `threshold`
- `xcode`
- `xml`

Machine-readable reporters are treated as compatibility-sensitive: JSON, SARIF,
XML, CSV, and Markdown should keep the shape expected by automation.

## Compatibility Model

The release gate is coverage-first. On the same inputs and options, `jscpd-rs`
must not miss duplicated source lines reported by upstream `jscpd`.

Exact clone pair identity, pair ordering, token totals, and some fragment
boundaries may differ while duplicated upstream source lines remain covered.
Extra Rust findings are allowed during convergence, but compatibility reports
keep them visible as `extra` findings.

This policy matters for multi-way clones: two implementations can pick
different clone pairs while still covering the same duplicated source ranges.

## Known First-Release Limits

These are intentional first-release limits:

- dynamic npm reporters, stores, listeners, and plugins are not loaded;
- unknown external reporter/store names keep upstream-style warnings where
  upstream continues;
- HTML reports are self-contained and practically compatible, not
  pixel-perfect;
- long-tail formats use a synchronized format registry plus generic native
  tokenization unless a fixture or public-repo gate proves a more specific
  tokenizer is needed;
- the Rust crate exposes a native Rust API, not the upstream JavaScript package
  API;
- current npm packaging is source-build: a Rust/Cargo toolchain is required
  during install until prebuilt platform packages are added.

## Compare On Your Repository

Run upstream and `jscpd-rs` with the same high-level options and compare the
generated JSON reports:

```bash
node jscpd/apps/jscpd/bin/jscpd src \
  --reporters json \
  --output /tmp/jscpd-upstream \
  --min-tokens 50 \
  --min-lines 5 \
  --exitCode 0

jscpd src \
  --reporters json \
  --output /tmp/jscpd-rs \
  --min-tokens 50 \
  --min-lines 5 \
  --exitCode 0
```

The repository scripts include the same coverage-first comparator used by the
release gates:

```bash
FORMAT=typescript MIN_TOKENS=50 MIN_LINES=5 STRICT=coverage \
  scripts/compat.sh /path/to/project
```

If `jscpd-rs` misses duplicated lines that upstream reports, that is a
compatibility bug. If `jscpd-rs` reports additional findings, include them in
the issue too; they help decide whether a tokenizer or boundary rule needs
tightening.

## CI Migration Pattern

Existing upstream-style CI:

```yaml
- run: npx jscpd src --reporters console,json --threshold 5 --exitCode 1
```

Cargo-based `jscpd-rs` CI:

```yaml
- uses: dtolnay/rust-toolchain@stable
- run: cargo install jscpd-rs --locked
- run: jscpd src --reporters console,json --threshold 5 --exitCode 1
```

npm/npx-based `jscpd-rs` CI:

```yaml
- uses: dtolnay/rust-toolchain@stable
- uses: actions/setup-node@v5
  with:
    node-version: 22
- run: npx jscpd-rs src --reporters console,json --threshold 5 --exitCode 1
```

The npm source-build package still needs Rust available during installation.
Prebuilt npm platform packages are the next packaging milestone; see the
[prebuilt binary distribution plan](prebuilt-binaries.md).
