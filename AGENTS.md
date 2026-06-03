# AGENTS.md

## Project Goal

This project is a high-performance Rust clone of
[`jscpd`](https://github.com/kucherenko/jscpd).

The goal is full practical compatibility with upstream `jscpd`: command-line
behavior, configuration formats, supported languages, reports, exit codes, and
integration workflows should match the reference implementation unless a
deliberate incompatibility is documented.

The upstream `jscpd` repository is kept in `jscpd/` as a git submodule and is the
primary reference for behavior.

Current first-release decisions are recorded in `docs/release-decisions.md`.
Follow that document for compatibility gate semantics, native-only runtime
strategy, reporter/store scope, long-tail format policy, and upstream-quirk
handling.

## Compatibility Policy

The primary compatibility gate is coverage-first parity: on the same inputs and
options, this Rust clone must not miss duplicate fragments reported by upstream
`jscpd`. Additional duplicates reported only by the Rust implementation are
allowed while the project is still converging, but they must remain visible in
compatibility reports as `extra` findings.

Exact 1:1 report parity is still valuable, but it is a quality metric rather
than the default blocking gate. Treat missing upstream fragments as blockers;
treat exact-pair ordering differences as diagnostics for multi-way duplicates.
Treat extra Rust duplicates as follow-up compatibility work unless they reveal a
clear false-positive regression.

## Engineering Principles

- Optimize for business value: faster development, reliable releases, simpler
  maintenance, and lower CI/compute cost. Code is a cost, not the product goal.
- Prefer the minimum sufficient change that solves the current problem and is
  easy to read, test, and change. Avoid future-proofing until a real workflow
  needs it.
- Build a fast Rust implementation first: performance is a core product goal,
  not an afterthought.
- Do not accept simplification, refactoring, or compatibility work that causes a
  sustained speed regression on the public benchmark suite. If a change touches
  discovery, tokenization, matching, reporting hot paths, or server snippet
  checks, rerun the relevant benchmark case before considering the core stable.
- Prefer battle-tested crates over custom code. Keep project-specific logic as
  small as practical.
- For JS/TS syntax tokenization, prefer Oxc-backed token processing over a
  hand-rolled lexer. Keep only the glue needed for jscpd-compatible filtering,
  positions, hashing, and reporting.
- Keep it simple. Use KIS: straightforward data flow, small modules, and minimal
  abstraction until real complexity requires it.
- Use SOTA libraries and algorithms where they materially improve correctness,
  performance, maintainability, or ecosystem compatibility.
- Match upstream behavior before improving it. Optimizations must not silently
  change user-visible semantics.
- Treat the upstream project as executable specification: compare behavior
  against `jscpd/` when implementing CLI flags, config parsing, tokenization,
  detection logic, and reporters.
- Avoid rewriting mature infrastructure from scratch: prefer existing crates for
  CLI parsing, config formats, globbing, ignore files, syntax/token processing,
  serialization, reporting formats, concurrency, and diagnostics.
- Keep dependencies intentional: choose widely used, maintained crates with clear
  APIs and acceptable compile-time/runtime costs.
- Before and after edits, check whether the diff can be smaller, simpler, or
  replaced by an existing mature tool without reducing clarity or safety.
- Add focused compatibility tests as features are ported. Prefer fixtures based
  on upstream behavior.
- Put black-box behavior tests that use the public API in `tests/`. Keep small
  private-helper tests next to the module they protect.
- Document intentional deviations from upstream in the relevant code, tests, or
  project documentation.
