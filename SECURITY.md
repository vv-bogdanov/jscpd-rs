# Security Policy

## Supported Versions

Security fixes are released for the latest published `jscpd-rs` version.
Upgrade to the newest npm package or Cargo crate before reporting an issue that
may already be fixed.

## Reporting A Vulnerability

Please report security issues privately through GitHub Security Advisories:

<https://github.com/vv-bogdanov/jscpd-rs/security/advisories/new>

Do not include exploit details in public issues, pull requests, discussions, or
social posts before coordinated disclosure. If GitHub private vulnerability
reporting is unavailable, open a minimal public issue that asks for a private
contact path and does not include reproduction details.

Include:

- affected version and install method (`npm`, `npx`, or Cargo);
- operating system and architecture;
- the command that triggered the issue;
- whether the issue affects local CLI scans, report generation, or the server.

Expected disclosure timeline:

- acknowledgment within 7 days;
- initial triage within 14 days;
- coordinated fix and disclosure target within 90 days for confirmed
  vulnerabilities, adjusted for severity and ecosystem impact.

Security fixes are released through Cargo, npm, and GitHub Releases. When a
vulnerability affects published packages, the advisory should include affected
versions, patched versions, severity, impact, workaround status, and credit for
the reporter when requested.

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
