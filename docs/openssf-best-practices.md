# OpenSSF Best Practices

`jscpd-rs` is registered in the OpenSSF Best Practices Badge service:
<https://www.bestpractices.dev/projects/13086>.

The OpenSSF Scorecard `CII-Best-Practices` check is not satisfied by a local
README claim. It queries the OpenSSF Best Practices Badge service for the
repository URL, so the remaining work is a maintainer-owned registration step.
Do not add the OpenSSF Best Practices badge to `README.md` until the project
entry reaches at least the passing level.

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

## Fast Passing Checklist

The public project page was 19% complete immediately after registration because
most fields were still `?`. The following fields can be marked in the OpenSSF
web UI using current repository evidence.

Mark `Met`:

- `description_good`, `interact`, `english`, `maintained`
- `contribution_requirements`
- `documentation_interface`
- `repo_interim`
- `version_unique`, `version_semver`, `version_tags`
- `release_notes_vulns`
- `report_tracker`, `report_responses`, `enhancement_responses`,
  `report_archive`
- `vulnerability_report_process`, `vulnerability_report_private`,
  `vulnerability_report_response`
- `build`, `build_common_tools`, `build_floss_tools`
- `test`, `test_invocation`, `test_most`, `test_continuous_integration`
- `test_policy`, `tests_are_added`, `tests_documented_added`
- `warnings`, `warnings_fixed`, `warnings_strict`
- `know_secure_design`, `know_common_errors`
- `delivery_unsigned`, `vulnerabilities_fixed_60_days`,
  `vulnerabilities_critical_fixed`, `no_leaked_credentials`
- `static_analysis`, `static_analysis_common_vulnerabilities`,
  `static_analysis_fixed`, `static_analysis_often`
- `dynamic_analysis`, `dynamic_analysis_unsafe`,
  `dynamic_analysis_enable_assertions`, `dynamic_analysis_fixed`

Mark `N/A` for the cryptographic criteria unless the project later adds
security-sensitive cryptographic functionality:

- `crypto_published`, `crypto_call`, `crypto_floss`, `crypto_keylength`,
  `crypto_working`, `crypto_weaknesses`, `crypto_pfs`,
  `crypto_password_storage`, `crypto_random`

Recommended evidence URLs:

- Project overview, install, usage, feedback, and contribution links:
  <https://github.com/vv-bogdanov/jscpd-rs#readme>
- Contribution and test policy:
  <https://github.com/vv-bogdanov/jscpd-rs/blob/main/CONTRIBUTING.md>
- Security reporting and analysis policy:
  <https://github.com/vv-bogdanov/jscpd-rs/blob/main/SECURITY.md>
- User-facing interface documentation:
  <https://github.com/vv-bogdanov/jscpd-rs/blob/main/docs/user-guide.md>
- Release checklist and gate evidence:
  <https://github.com/vv-bogdanov/jscpd-rs/blob/main/docs/release-checklist.md>
- GitHub Actions workflows:
  <https://github.com/vv-bogdanov/jscpd-rs/tree/main/.github/workflows>
