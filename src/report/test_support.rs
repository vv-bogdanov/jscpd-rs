use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use crate::cli::Options;
use crate::detector::{
    BlamedLine, CloneMatch, DetectionResult, FormatStatistic, Fragment, StatisticRow,
};
use crate::report::write_reports;
use crate::tokenizer::Location;

pub(super) fn make_test_statistics() -> crate::detector::Statistics {
    let mut formats = HashMap::new();
    formats.insert(
        "javascript".to_string(),
        FormatStatistic {
            sources: HashMap::new(),
            total: test_statistic_row(),
        },
    );
    crate::detector::Statistics {
        total: test_statistic_row(),
        formats,
    }
}

fn test_statistic_row() -> StatisticRow {
    StatisticRow {
        sources: 2,
        lines: 20,
        tokens: 100,
        clones: 1,
        duplicated_lines: 5,
        duplicated_tokens: 30,
        percentage: 25.0,
        percentage_tokens: 30.0,
        new_duplicated_lines: 0,
        new_clones: 0,
    }
}

pub(super) fn make_test_clone(source_a: &str, source_b: &str) -> CloneMatch {
    CloneMatch {
        format: "javascript".to_string(),
        duplication_a: Fragment {
            source_id: source_a.to_string(),
            start: location(2, 3, 0),
            end: location(5, 1, 18),
            range: [0, 18],
            blame: None,
        },
        duplication_b: Fragment {
            source_id: source_b.to_string(),
            start: location(8, 1, 0),
            end: location(11, 1, 18),
            range: [0, 18],
            blame: None,
        },
        tokens: 6,
    }
}

pub(super) fn make_test_result_with_clone(source_a: &str, source_b: &str) -> DetectionResult {
    let mut source_contents = HashMap::new();
    source_contents.insert(source_a.to_string(), "alpha <beta> ]]>\n".to_string());
    source_contents.insert(source_b.to_string(), "alpha & beta\nxxxx\n".to_string());

    DetectionResult {
        clones: vec![make_test_clone(source_a, source_b)],
        skipped_clones: Vec::new(),
        statistics: make_test_statistics(),
        sources: Vec::new(),
        source_contents,
    }
}

pub(super) fn temp_output(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("jscpd-rs-{label}-{}-{nonce}", std::process::id()))
}

pub(super) fn write_test_report(reporter: &str, label: &str, path: &[&str]) -> String {
    let output = write_test_report_output(reporter, label);
    let report_path = path
        .iter()
        .fold(output.clone(), |path, segment| path.join(segment));
    let report = std::fs::read_to_string(report_path).unwrap();
    let _ = std::fs::remove_dir_all(output);
    report
}

pub(super) fn write_test_report_output(reporter: &str, label: &str) -> PathBuf {
    let output = temp_output(label);
    let options = Options {
        output: output.clone(),
        reporters: vec![reporter.to_string()],
        silent: true,
        ..Options::default()
    };
    let result = make_test_result_with_clone("src/a.js", "src/b.js");

    write_reports(&result, &options).unwrap();
    output
}

pub(super) fn single_line_blame(
    line: &str,
    rev: &str,
    author: &str,
    date: &str,
) -> BTreeMap<String, BlamedLine> {
    [(
        line.to_string(),
        BlamedLine {
            rev: rev.to_string(),
            author: author.to_string(),
            date: date.to_string(),
            line: line.to_string(),
        },
    )]
    .into_iter()
    .collect()
}

fn location(line: usize, column: usize, position: usize) -> Location {
    Location {
        line,
        column,
        position,
    }
}
