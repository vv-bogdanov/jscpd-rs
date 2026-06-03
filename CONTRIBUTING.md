# Contributing

`jscpd-rs` is a fast Rust implementation of the common upstream `jscpd`
workflow. The project values practical compatibility, speed, and a small
maintenance surface.

## Useful Reports

The most valuable issues include:

- an upstream duplicate that `jscpd-rs` misses on the same input and options;
- an npm install problem on a supported platform;
- a report-format mismatch that breaks CI or tooling;
- a public repository that should be part of the benchmark suite;
- a clear false positive that produces noisy duplicate reports.

Please include the command, `jscpd-rs --version`, OS/architecture, and a small
repro fixture when possible.

## Contribution Requirements

Pull requests should be small, focused, and covered by the narrowest useful
tests. Changes that add or alter user-visible behavior should include
behavioral tests or compatibility fixtures. Changes in hot paths should keep
the public benchmark suite from regressing.

Before sending a pull request:

- run `cargo fmt`;
- run `cargo test` or a narrower test command that covers the change;
- run `cargo clippy --all-targets -- -D warnings` when changing Rust logic;
- update docs when CLI flags, reports, install behavior, or public APIs change.

New functionality should include automated tests in the same change unless the
feature is intentionally experimental and the risk is documented. Compatibility
work should prefer fixtures that compare against upstream `jscpd`.

## Development Checks

Before sending a change, run the narrowest check that covers it:

```bash
cargo test
```

For packaging changes:

```bash
scripts/npm-package-check.sh
```

For release-level confidence:

```bash
scripts/release-candidate.sh
```

`scripts/release-gate.sh` is the default CI gate. It runs formatting checks,
tests, missing public-doc checks, shell/action linting, package/install smoke,
and upstream compatibility checks. Release-candidate flows also run clippy with
warnings denied, coverage, supply-chain checks, the full compatibility matrix,
and public benchmarks.

## Compatibility Rule

The default gate is coverage-first: for the same inputs and options, `jscpd-rs`
must not miss duplicated source lines reported by upstream `jscpd`. Extra Rust
findings stay visible as compatibility work.

Prefer small changes, existing crates, and direct code. Avoid adding new
abstractions or custom infrastructure unless they clearly reduce risk or
maintenance cost.
