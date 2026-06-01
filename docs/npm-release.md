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
  if the current version is already published, this dry-run is skipped with an
  explicit message;
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

## Trusted Publishing

Trusted Publishing is the preferred npm path for CI/CD publishing. npm trusts a
specific GitHub Actions workflow through OIDC, so the workflow can publish
without a long-lived npm token. For public packages published from public
repositories, npm also generates provenance attestations automatically.

This repository provides a publish workflow:

```text
.github/workflows/npm-publish.yml
```

The workflow runs automatically when a non-draft, non-prerelease GitHub Release
is published. It checks out the release tag, verifies that the tag matches the
`package.json` version, verifies that the exact npm version is not already
published, runs `scripts/npm-package-check.sh`, and then runs
`npm publish --access public`.

It can also be run manually from GitHub Actions as a fallback by entering
`jscpd-rs` in the confirmation input.

Configure npm:

1. Open the `jscpd-rs` package settings on npmjs.com.
2. Go to **Trusted Publisher**.
3. Select **GitHub Actions**.
4. Use these values:
   - Organization or user: `vv-bogdanov`
   - Repository: `jscpd-rs`
   - Workflow filename: `npm-publish.yml`
   - Environment name: leave empty
   - Allowed actions: `npm publish`

Then publish automatically from GitHub:

1. Update `Cargo.toml` and `package.json` to the same version.
2. Run the release gates.
3. Create and publish a GitHub Release with tag `vX.Y.Z`.
4. GitHub Actions will run `npm-publish` from that release tag.

Manual fallback:

1. Open GitHub Actions.
2. Select the `npm-publish` workflow.
3. Click **Run workflow** on `main`.
4. Enter `jscpd-rs` for `package_name`.
5. Run the workflow.

If npm does not allow Trusted Publishing configuration before the first package
version exists, publish the first version with either interactive 2FA or a
short-lived granular token with bypass 2FA enabled, then immediately configure
Trusted Publishing for future releases.

After Trusted Publishing is verified, revoke temporary npm tokens and consider
setting the npm package publishing access to disallow traditional tokens.
