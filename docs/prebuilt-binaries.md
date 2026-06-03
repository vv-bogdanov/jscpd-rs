# Prebuilt Binary Distribution

`jscpd-rs` is a native Rust CLI. Starting with `jscpd-rs@0.1.4`, npm
distribution uses a small main package plus platform-specific optional
packages without install-time build scripts. The original `0.1.0` npm package
was source-build only.

The main `jscpd-rs` package keeps the public bin names: `jscpd-rs`, `jscpd`,
and `jscpd-server`.

## Runtime Behavior

- The JS shim first looks for a matching prebuilt optional package.
- If a prebuilt package is installed, the shim runs its native binary directly.
- If no prebuilt package exists for the platform, the shim exits with a clear
  message that points users to `cargo install jscpd-rs --locked`.

Avoid default postinstall downloads or builds. Optional platform packages are
more reproducible, work better with npm mirrors and lockfiles, and avoid
surprising code execution during install.

## Platform Packages

The target matrix is defined in `npm/prebuilt-targets.json`.

| Package | Rust target | Runner |
| --- | --- | --- |
| `jscpd-rs-linux-x64-gnu` | `x86_64-unknown-linux-gnu` | `ubuntu-24.04` |
| `jscpd-rs-linux-arm64-gnu` | `aarch64-unknown-linux-gnu` | `ubuntu-22.04-arm` |
| `jscpd-rs-darwin-x64` | `x86_64-apple-darwin` | `macos-15-intel` |
| `jscpd-rs-darwin-arm64` | `aarch64-apple-darwin` | `macos-15` |
| `jscpd-rs-win` | `x86_64-pc-windows-msvc` | `windows-2025-vs2026` |

Consider Linux musl and Windows arm64 only after install data or user reports
show demand.

## Package Checks

`scripts/npm-package-check.sh` verifies:

- the root npm version matches `Cargo.toml`;
- `optionalDependencies` exactly match `npm/prebuilt-targets.json`;
- every optional platform dependency uses the same version as the main package;
- the main package does not declare install lifecycle scripts;
- the main npm package contains the runtime shim and target metadata;
- installing without optional prebuilt dependencies succeeds without running
  build scripts, then the CLI fails with the expected install hint;
- a locally generated platform package works without Cargo;
- `npx` works when local main and platform tarballs are installed together.

## Release Workflow

The GitHub Release workflow in `.github/workflows/npm-publish.yml` publishes in
this order:

1. Run `scripts/release-candidate.sh`, including the full compatibility
   matrix, public speed gate, and core coverage gate.
2. Verify that the GitHub Release tag matches `package.json`.
3. Build platform packages in a native runner matrix.
4. Smoke-test `jscpd --version` and `jscpd-server --version` for each package.
5. Publish the platform packages.
6. Verify every optional platform package is published for the release version.
7. Run `scripts/npm-package-check.sh`.
8. Publish the main `jscpd-rs` package.

The main npm package is intentionally blocked when any configured platform
package fails or is missing. Manual target-only reruns must set
`publish_main=false`; the main package should not be published with an
unsupported platform-package set.

The workflow publishes with npm provenance enabled. It is designed for Trusted
Publishing, but `NPM_TOKEN` can be kept as a temporary bootstrap fallback for
new platform packages if npm does not allow Trusted Publisher setup before the
first version exists.

## npm Setup

Configure Trusted Publishing for each npm package with:

- Organization or user: `vv-bogdanov`
- Repository: `jscpd-rs`
- Workflow filename: `npm-publish.yml`
- Environment name: leave empty
- Allowed actions: `npm publish`

Packages:

- `jscpd-rs`
- `jscpd-rs-linux-x64-gnu`
- `jscpd-rs-linux-arm64-gnu`
- `jscpd-rs-darwin-x64`
- `jscpd-rs-darwin-arm64`
- `jscpd-rs-win`

If a temporary npm token is used for the first platform-package bootstrap,
revoke it after Trusted Publishing succeeds.
