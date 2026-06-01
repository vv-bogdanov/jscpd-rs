# Npm Release Preparation

Current npm readiness snapshot:

- Date: 2026-06-01.
- Baseline commit: `e343350`; rerun `scripts/npm-package-check.sh` on the
  exact checkout before npm publish.
- GitHub Actions `release-gate`: passed on run `26743839098`.
- Local checks passed: `cargo test`, `scripts/package-check.sh`,
  `scripts/npm-package-check.sh`, local `npx --no-install` smoke for
  `jscpd-rs`, `jscpd`, and `jscpd-server`.
- Package name status: `npm view jscpd-rs version` returned `E404`.
- Packed artifact audit: `jscpd-rs-0.1.0.tgz`, 96 files, about 169 KiB packed,
  about 708 KiB unpacked.

The first npm package is `jscpd-rs`. It exposes these bin commands:

- `jscpd-rs`: primary `npx jscpd-rs` entrypoint, runs the native `jscpd` CLI.
- `jscpd`: installed alias for the native `jscpd` CLI.
- `jscpd-server`: installed alias for the native server binary.

The package is intentionally source-build for the first release candidate:
`postinstall` runs `cargo build --release --locked --bin jscpd --bin
jscpd-server` inside the installed npm package. This keeps the npm path simple
and verifiable before publication. Users installing from npm need Node, npm, and
a Rust/Cargo toolchain. Prebuilt platform packages can be added later without
changing the CLI behavior.

Local verification:

```bash
scripts/npm-package-check.sh
```

That script verifies:

- `package.json` version matches `Cargo.toml`;
- `npm pack` includes the expected Rust source and npm shim files;
- `npm pack` includes the advertised `skills/` files used by the terminal tip;
- forbidden paths such as `jscpd/`, `target/`, `report/`, `scripts/`, and
  `node_modules/` are not packed;
- `npm publish --dry-run --json` succeeds without publishing;
- installing the packed tarball without Cargo fails with the expected Rust
  toolchain hint;
- a local npm install exposes working `jscpd-rs`, `jscpd`, and `jscpd-server`
  bin commands;
- `npx --package <local-tarball> jscpd-rs --version` works.

Before actual publication, run:

```bash
git status --short
npm whoami
scripts/npm-package-check.sh
npm view jscpd-rs version
```

`npm view` should return `E404` for the first publication, or the package must
already be owned by this project. Do not run `npm publish` until explicit
release approval.

Do not use `scripts/prepublish-check.sh` as the npm-only gate after the Cargo
crate and GitHub tag have already been published: that script is the full
Cargo/GitHub first-publication gate and intentionally checks that the crate name
and release tag are still available.

If the package name is still free and the npm account is logged in:

```bash
scripts/npm-package-check.sh
npm publish --access public
```

If the account requires a one-time password for publish:

```bash
npm publish --access public --otp 123456
```
