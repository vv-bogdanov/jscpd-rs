# Prebuilt Binary Distribution Plan

`jscpd-rs` is already a native Rust CLI, but the current npm package builds the
native binaries from source during `postinstall`. That is acceptable for the
bootstrap release and awkward for broad npm adoption: users need Cargo, native
build tools, and extra install time before they can run a duplicate-code scan.

The recommended npm distribution model is a small main package plus
platform-specific optional packages, similar to other native Node CLIs.

## Recommended Shape

- Keep `jscpd-rs` as the main package with the public bin names:
  `jscpd-rs`, `jscpd`, and `jscpd-server`.
- Publish platform packages with exactly one prebuilt binary pair each.
- Add the platform packages as `optionalDependencies` of `jscpd-rs`.
- Use `os`, `cpu`, and where supported `libc` metadata in platform packages so
  npm installs only the matching package.
- Resolve the platform package at runtime from the existing JS shims.
- Keep source-build fallback for unsupported platforms and local development.
- Fail with a direct install hint when neither a prebuilt package nor Cargo is
  available.

Avoid default postinstall downloads from GitHub Releases. Optional platform
packages are more reproducible, work better with npm mirrors and lockfiles, and
avoid surprising network access during install.

## Initial Target Matrix

Start with the platforms most likely to cover CI and developer machines:

| Package target | Notes |
| --- | --- |
| Linux x64 GNU | Highest-priority GitHub Actions and server target. |
| Linux arm64 GNU | Common in ARM CI runners and cloud instances. |
| macOS arm64 | Apple Silicon default. |
| macOS x64 | Intel macOS fallback. |
| Windows x64 MSVC | Main Windows developer target. |

Consider Linux musl and Windows arm64 only after install data or user reports
show demand.

## Current Install Friction

- npm installs require Cargo and Rust `1.93+`.
- Source builds are slow compared with unpacking a binary package.
- Windows users may need the MSVC build tools before install succeeds.
- macOS users may need Xcode command-line tools.
- Ephemeral CI jobs pay the compile cost unless they cache Cargo artifacts.
- `npx jscpd-rs` is convenient but still triggers npm install/build on a cold
  cache.
- The `jscpd` bin alias intentionally shadows upstream `jscpd`; users should
  check `jscpd --version` during migration.

## Release Workflow

The future release workflow should remain GitHub Release driven:

1. Build release binaries in a matrix for the target platforms.
2. Smoke-test `jscpd --version`, `jscpd --help`, and `jscpd-server --version`
   on each native runner where possible.
3. Publish platform npm packages with the same version as the main package.
4. Publish the main `jscpd-rs` npm package with optional dependencies pointing
   at those exact versions.
5. Keep `scripts/npm-package-check.sh` testing both prebuilt resolution and
   source-build fallback.

This should happen before broad npm promotion. Cargo users can keep using
`cargo install jscpd-rs --locked`; Rust source builds are normal in that
ecosystem.
