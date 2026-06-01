#![doc(html_root_url = "https://docs.rs/jscpd-rs/0.1.2")]

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
//!
//! # Compatibility Model
//!
//! The release gate is coverage-first: for the same inputs and options, this
//! crate must not miss duplicated source lines reported by upstream `jscpd`.
//! Extra Rust findings remain visible in compatibility reports while the
//! implementation converges on exact parity.
//!
//! The first release intentionally keeps the detector native-only. Dynamic npm
//! reporters, stores, listeners, and plugins are not loaded by this crate.
//!
//! See the
//! [README](https://github.com/vv-bogdanov/jscpd-rs#readme) and
//! [User Guide](https://github.com/vv-bogdanov/jscpd-rs/blob/main/docs/user-guide.md)
//! for CLI, configuration, reporter, server, and CI examples.

pub mod app;
pub mod blame;
pub mod cli;
pub mod detector;
pub mod files;
pub mod formats;
pub mod report;
pub mod server;
pub mod tokenizer;
pub mod verbose;

use std::{ffi::OsString, path::Path};

use anyhow::Result;

pub use app::{JscpdOutcome, jscpd, jscpd_with_exit_callback, run_cli_args};
pub use cli::{FormatMappings, Options};
pub use detector::{
    CloneMatch, DetectionResult, Detector, MemoryStore, MemoryStoreError, Statistic, StatisticRow,
    Statistics,
};
pub use files::SourceFile;
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
/// The first release keeps the registry aligned with upstream `jscpd`; high
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    const DUPLICATE_JS: &str = "const alpha = 1;\nconst beta = 2;\nconst gamma = alpha + beta;\n";

    fn fixture_options(path: &str) -> Options {
        Options {
            paths: vec![PathBuf::from(path)],
            reporters: Vec::new(),
            silent: true,
            no_tips: true,
            min_tokens: 20,
            min_lines: 3,
            max_size_bytes: 1024 * 1024,
            ..Options::default()
        }
    }

    fn in_memory_options() -> Options {
        Options {
            reporters: Vec::new(),
            silent: true,
            no_tips: true,
            min_tokens: 5,
            min_lines: 2,
            ..Options::default()
        }
    }

    fn javascript_source(source_id: &str, content: &str) -> SourceFile {
        SourceFile {
            source_id: source_id.to_string(),
            format: "javascript".to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn public_api_detects_clones_from_paths() {
        let options = fixture_options("jscpd/fixtures/clike/file2.c");

        let clones = detect_clones(&options).expect("detect clones");

        assert_eq!(clones.len(), 1);
        assert_eq!(clones[0].duplication_a.start.line, 18);
        assert_eq!(clones[0].duplication_b.start.line, 8);
    }

    #[test]
    fn public_api_returns_statistics() {
        let options = fixture_options("jscpd/fixtures/clike/file2.c");

        let result = detect_clones_and_statistics(&options).expect("detect with statistics");

        assert_eq!(result.clones.len(), 1);
        assert_eq!(result.statistics.total.clones, 1);
        assert_eq!(result.statistics.total.sources, 1);
    }

    #[test]
    fn public_api_statistic_alias_matches_upstream_name() {
        let options = fixture_options("jscpd/fixtures/clike/file2.c");

        let result = detect_clones_and_statistic(&options).expect("detect with statistic alias");

        assert_eq!(result.clones.len(), 1);
        assert_eq!(result.statistics.total.clones, 1);
    }

    #[test]
    fn public_api_exposes_default_options() {
        let options = get_default_options();

        assert_eq!(options.min_lines, 5);
        assert_eq!(options.min_tokens, 50);
        assert_eq!(options.max_lines, 1000);
        assert_eq!(options.max_size_bytes, 100 * 1024);
        assert_eq!(options.reporters, vec!["console"]);
        assert!(options.cache);
        assert!(options.gitignore);
    }

    #[test]
    fn public_api_parses_options_from_args() {
        let options = get_options_from_args([
            "jscpd",
            "fixtures",
            "--format",
            "javascript,typescript",
            "--reporters",
            "json",
            "--min-tokens",
            "7",
            "--min-lines",
            "2",
            "--max-size",
            "1mb",
            "--noTips",
        ])
        .expect("parse options from argv");

        let expected_formats = vec!["javascript".to_string(), "typescript".to_string()];
        assert_eq!(options.paths, vec![PathBuf::from("fixtures")]);
        assert_eq!(
            options.format_order.as_deref(),
            Some(expected_formats.as_slice())
        );
        assert_eq!(options.reporters, vec!["json"]);
        assert_eq!(options.min_tokens, 7);
        assert_eq!(options.min_lines, 2);
        assert_eq!(options.max_size_bytes, 1024 * 1024);
        assert!(options.no_tips);
    }

    #[test]
    fn public_api_arg_parser_preserves_runtime_option_errors() {
        let error = get_options_from_args(["jscpd", "--mode", "zzz", "."]).unwrap_err();

        assert_eq!(error.to_string(), "Mode zzz does not supported yet.");
    }

    #[test]
    fn public_api_exposes_supported_formats() {
        let formats = get_supported_formats();

        assert_eq!(formats.len(), 223);
        assert!(formats.contains(&"javascript"));
        assert!(formats.contains(&"typescript"));
        assert!(formats.contains(&"rust"));
    }

    #[test]
    fn public_api_resolves_format_by_file() {
        assert_eq!(
            get_format_by_file("src/index.mts").as_deref(),
            Some("typescript")
        );
        assert_eq!(
            get_format_by_file("src/component.vue").as_deref(),
            Some("vue")
        );
    }

    #[test]
    fn public_api_resolves_format_by_custom_mappings() {
        let formats_exts = FormatMappings::from_pairs(vec![("custom", vec!["foo"])]);
        let formats_names = FormatMappings::from_pairs(vec![("makefile", vec!["Buildfile"])]);

        assert_eq!(
            get_format_by_file_with_mappings("demo.foo", &formats_exts, &formats_names).as_deref(),
            Some("custom")
        );
        assert_eq!(
            get_format_by_file_with_mappings("Buildfile", &formats_exts, &formats_names).as_deref(),
            Some("makefile")
        );
        assert_eq!(
            get_format_by_file_with_mappings("src/index.ts", &formats_exts, &formats_names),
            None
        );
    }

    #[test]
    fn public_api_detects_from_in_memory_sources() {
        let files = vec![
            javascript_source("snippet.js", DUPLICATE_JS),
            javascript_source("src/match.js", DUPLICATE_JS),
        ];

        let result = detect_source_files(files, &in_memory_options());

        assert_eq!(result.clones.len(), 1);
        assert_eq!(result.statistics.total.sources, 2);
    }

    #[test]
    fn public_api_exposes_streaming_detector() {
        let mut detector = Detector::new(in_memory_options());

        assert!(
            detector
                .detect("first.js", DUPLICATE_JS, "javascript")
                .is_empty()
        );
        let clones = detector.detect("second.js", DUPLICATE_JS, "javascript");

        assert_eq!(clones.len(), 1);
        assert_eq!(detector.sources().len(), 2);
        assert!(
            clones[0].duplication_a.source_id == "second.js"
                || clones[0].duplication_b.source_id == "second.js"
        );
    }

    #[test]
    fn public_api_exposes_statistic_collector() {
        let result = detect_source_files(
            vec![
                javascript_source("first.js", DUPLICATE_JS),
                javascript_source("second.js", DUPLICATE_JS),
            ],
            &in_memory_options(),
        );
        let mut statistic = Statistic::new();

        statistic.match_source("first.js", "javascript", 3, 42);
        statistic.match_source("second.js", "javascript", 3, 42);
        statistic.clone_found(&result.clones[0]);

        let stats = statistic.get_statistic();
        assert_eq!(stats.total.sources, 2);
        assert_eq!(stats.total.clones, 1);
        assert_eq!(stats.formats["javascript"].sources["first.js"].sources, 1);
    }

    #[test]
    fn public_api_exposes_memory_store() {
        let mut store = MemoryStore::new();

        store.namespace("javascript");
        assert_eq!(*store.set("hash", 7usize), 7);
        assert_eq!(*store.get("hash").expect("stored value"), 7);
        store.namespace("typescript");
        let error = store.get("hash").unwrap_err();

        assert_eq!(error.namespace(), "typescript");
        assert_eq!(error.key(), "hash");
        assert_eq!(store.len(), 1);
        store.close();
        assert!(store.is_empty());
    }
}
