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

## Compatibility Rule

The default gate is coverage-first: for the same inputs and options, `jscpd-rs`
must not miss duplicated source lines reported by upstream `jscpd`. Extra Rust
findings stay visible as compatibility work.

Prefer small changes, existing crates, and direct code. Avoid adding new
abstractions or custom infrastructure unless they clearly reduce risk or
maintenance cost.
