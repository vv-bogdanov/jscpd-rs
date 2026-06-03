# OpenSSF Best Practices

`jscpd-rs` should apply for the OpenSSF Best Practices badge after the public
release workflow settles.

The OpenSSF Scorecard `CII-Best-Practices` check is not satisfied by a local
README claim. It queries the OpenSSF Best Practices Badge service for the
repository URL, so the remaining work is a maintainer-owned registration step.

Use this evidence when filling the project profile:

- Public source repository: <https://github.com/vv-bogdanov/jscpd-rs>
- License: MIT, recorded in `LICENSE` and Cargo/npm metadata.
- Public issue tracker and pull requests: GitHub Issues and Pull Requests.
- Security reporting: `SECURITY.md` and GitHub private advisories.
- CI: GitHub Actions release gate, CodeQL, Scorecard, Socket, and fuzz smoke.
- Releases: GitHub Releases, crates.io, and npm with npm provenance.
- Documentation: `README.md`, docs in `docs/`, docs.rs, and npm README.
- Contribution process: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and
  `.github/CODEOWNERS`.

After registration, add the official badge URL to `README.md` only when the
project entry exists and shows the real in-progress or passing status.
