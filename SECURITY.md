# Security Policy

## Supported Versions

Security fixes are released for the latest published `jscpd-rs` version.
Upgrade to the newest npm package or Cargo crate before reporting an issue that
may already be fixed.

## Reporting A Vulnerability

Please report security issues privately by using GitHub Security Advisories for
this repository when available. If that is not available, open a minimal public
issue that asks for a private contact path without including exploit details.

Include:

- affected version and install method (`npm`, `npx`, or Cargo);
- operating system and architecture;
- the command that triggered the issue;
- whether the issue affects local CLI scans, report generation, or the server.

## Supply Chain Notes

- npm releases are published from GitHub Actions with npm provenance enabled.
- The npm package uses optional platform packages for native binaries.
- The main npm package does not run install-time build scripts.
- Unsupported npm platforms should install through Cargo:

```bash
cargo install jscpd-rs --locked
```

`jscpd-rs` is a native code package by design. Treat the published npm
provenance, GitHub release, Cargo crate, and repository source as the trust
chain for release verification.
