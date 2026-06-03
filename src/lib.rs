#![doc(html_root_url = "https://docs.rs/jscpd-rs/0.1.10")]

//! Native Rust API for `jscpd-rs`, a 50x+ faster duplicate-code detector for
//! local development and CI/CD.
//!
//! `jscpd-rs` scans a codebase, finds copy-paste fragments across files, writes
//! console, JSON, SARIF, HTML, XML, CSV, Markdown, badge, and Xcode reports,
//! and can fail a build when duplication crosses a configured threshold.
//!
//! It is a native Rust implementation of the common
//! [`jscpd`](https://github.com/kucherenko/jscpd) command-line workflow:
//! upstream-style CLI flags, `.jscpd.json` and `package.json#jscpd`
//! configuration, report formats, exit-code behavior, Git blame, and server
//! snippet checks. The current public benchmark suite records 50x+ speedups on
//! pinned React, Next.js, and Prometheus cases while using a coverage-first
//! compatibility gate against upstream `jscpd`.
//!
//! This crate exposes the same detector core used by the `jscpd` and
//! `jscpd-server` binaries: option parsing, file discovery, tokenization,
//! duplicate detection, statistics, and in-memory source checks.
//!
//! # Quick Start
//!
//! Scan paths using the same option model as the CLI:
//!
//! ```no_run
//! use std::path::PathBuf;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut options = jscpd_rs::get_default_options();
//! options.paths = vec![PathBuf::from("src")];
//! options.reporters.clear();
//! options.silent = true;
//!
//! let result = jscpd_rs::detect_clones_and_statistics(&options)?;
//! println!("{} clones", result.clones.len());
//! # Ok(())
//! # }
//! ```
//!
//! Check prepared in-memory sources without touching the filesystem:
//!
//! ```
//! let mut options = jscpd_rs::get_default_options();
//! options.reporters.clear();
//! options.min_lines = 2;
//! options.min_tokens = 5;
//!
//! let files = vec![
//!     jscpd_rs::SourceFile {
//!         source_id: "a.js".to_string(),
//!         format: "javascript".to_string(),
//!         content: "const a = 1;\nconst b = 2;\nconst c = a + b;\n".to_string(),
//!     },
//!     jscpd_rs::SourceFile {
//!         source_id: "b.js".to_string(),
//!         format: "javascript".to_string(),
//!         content: "const a = 1;\nconst b = 2;\nconst c = a + b;\n".to_string(),
//!     },
//! ];
//!
//! let result = jscpd_rs::detect_source_files(files, &options);
//! assert!(!result.clones.is_empty());
//! ```
//!
//! # Main Entry Points
//!
//! - [`get_options_from_args`] parses upstream-style CLI arguments into
//!   [`Options`].
//! - [`detect_clones`] and [`detect_clones_and_statistics`] run discovery,
//!   tokenization, duplicate detection, statistics, and optional Git blame.
//! - [`detect_source_files`] runs detection against caller-provided
//!   [`SourceFile`] values and is the best entry point for editors, servers,
//!   and tests.
//! - [`Tokenizer`] exposes the native token map generator used by the detector.
//! - [`Detector`] and [`MemoryStore`] provide Rust counterparts for the main
//!   upstream core classes.
//! - [`jscpd`] and [`jscpd_with_exit_callback`] provide an embeddable argv
//!   runner similar to upstream `jscpd(argv, exitCallback?)`.
//! - [`serve`] and [`serve_with_working_directory`] start the native REST/MCP
//!   server used by the `jscpd-server` binary.
//!
//! # Compatibility Model
//!
//! The release gate is coverage-first: for the same inputs and options, this
//! crate must not miss duplicated source lines reported by upstream `jscpd`.
//! Extra Rust findings remain visible in compatibility reports while the
//! implementation converges on exact parity.
//!
//! The current 0.x line intentionally keeps the detector native-only. Dynamic npm
//! reporters, stores, listeners, and plugins are not loaded by this crate.
//!
//! See the
//! [README](https://github.com/vv-bogdanov/jscpd-rs#readme) and
//! [User Guide](https://github.com/vv-bogdanov/jscpd-rs/blob/main/docs/user-guide.md)
//! for CLI, configuration, reporter, server, and CI examples.

mod app;
mod blame;
mod cli;
mod detector;
mod files;
mod formats;
mod report;
mod server;
mod tokenizer;
mod verbose;

use std::{ffi::OsString, path::Path};

use anyhow::Result;

pub use app::{
    JscpdOutcome, jscpd, jscpd_with_exit_callback, run_cli_args, run_current_process,
    upstream_stdout_error,
};
pub use cli::{Cli, ExitCode, FormatMappings, Mode, Options};
pub use detector::{
    BlamedLine, BlamedLines, CloneMatch, DetectionResult, Detector, Fragment, MemoryStore,
    MemoryStoreError, SkippedClone, SourceSummary, Statistic, StatisticRow, Statistics,
};
pub use files::SourceFile;
pub use report::ThresholdExceeded;
pub use server::{serve, serve_with_working_directory, server_working_directory};
pub use tokenizer::{DetectionToken, Location, SourceTokenMap, TokenMap, Tokenizer};

/// Return the upstream-compatible default option set.
///
/// The defaults match the CLI defaults used by the `jscpd` binary: all
/// supported formats, `min_lines = 5`, `min_tokens = 50`, `max_lines = 1000`,
/// `max_size = 100kb`, Git ignore handling enabled, and the console reporter
/// selected.
pub fn get_default_options() -> Options {
    Options::default()
}

/// Parse upstream-style command-line arguments into normalized [`Options`].
///
/// The first argument should be the binary name, just like `std::env::args`.
/// This is useful for native integrations that want the same option semantics
/// as the CLI without spawning a process.
pub fn get_options_from_args<I, T>(args: I) -> Result<Options>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    Options::from_args(args)
}

/// Return the names of all formats known to the synchronized format registry.
///
/// The current 0.x line keeps the registry aligned with upstream `jscpd`; high
/// volume JS/TS formats use native Oxc-backed tokenization and long-tail
/// formats use the generic native tokenizer unless promoted by compatibility
/// evidence.
pub fn get_supported_formats() -> Vec<&'static str> {
    formats::supported_formats()
}

/// Resolve a source format from a path using the built-in extension and
/// filename registry.
pub fn get_format_by_file(path: impl AsRef<Path>) -> Option<String> {
    get_format_by_file_with_mappings(path, &FormatMappings::default(), &FormatMappings::default())
}

/// Resolve a source format from a path with caller-provided extension and
/// filename mappings.
///
/// This mirrors the CLI `--formats-exts` and `--formats-names` options.
pub fn get_format_by_file_with_mappings(
    path: impl AsRef<Path>,
    formats_exts: &FormatMappings,
    formats_names: &FormatMappings,
) -> Option<String> {
    formats::format_for_path(path.as_ref(), formats_exts, formats_names).map(str::to_string)
}

/// Detect clones from files discovered through [`Options::paths`].
///
/// This is the compact path-based API when callers only need clone matches and
/// not the full statistics object.
pub fn detect_clones(options: &Options) -> Result<Vec<CloneMatch>> {
    Ok(detect_clones_and_statistics(options)?.clones)
}

/// Upstream-named alias for [`detect_clones_and_statistics`].
///
/// The singular `statistic` spelling is kept for callers porting from upstream
/// JavaScript APIs and examples.
pub fn detect_clones_and_statistic(options: &Options) -> Result<DetectionResult> {
    detect_clones_and_statistics(options)
}

/// Detect clones and return both clone matches and aggregate statistics.
///
/// This entry point performs ignore-aware file discovery from [`Options::paths`]
/// before delegating to the native detector. Use [`detect_source_files`] when
/// the caller already has source contents in memory.
pub fn detect_clones_and_statistics(options: &Options) -> Result<DetectionResult> {
    let files = files::discover(options)?;
    Ok(detect_source_files(files, options))
}

/// Detect clones in prepared in-memory sources.
///
/// This is the lowest-friction API for editor integrations, tests, snippets,
/// and services that already own source contents. The `format` field on each
/// [`SourceFile`] should contain one of the names returned by
/// [`get_supported_formats`].
pub fn detect_source_files(files: Vec<SourceFile>, options: &Options) -> DetectionResult {
    let mut result = detector::detect(files, options);
    if options.blame {
        blame::apply_blame(&mut result);
    }
    result
}
