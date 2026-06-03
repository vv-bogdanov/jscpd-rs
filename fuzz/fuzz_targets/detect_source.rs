#![no_main]

use std::collections::HashSet;

use jscpd_rs::{Options, SourceFile, detect_source_files};
use libfuzzer_sys::fuzz_target;

const FORMATS: &[&str] = &[
    "javascript",
    "typescript",
    "jsx",
    "tsx",
    "rust",
    "go",
    "python",
    "markup",
    "markdown",
    "css",
    "json",
    "yaml",
];

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    let (format, content) = split_input(input);
    let options = Options {
        formats: Some(HashSet::from([format.to_string()])),
        max_lines: usize::MAX,
        max_size_bytes: u64::MAX,
        min_lines: 1,
        min_tokens: 3,
        reporters: Vec::new(),
        ..Options::default()
    };
    let source = SourceFile {
        source_id: format!("fuzz.{format}"),
        format: format.to_string(),
        content: content.to_string(),
    };

    let _ = detect_source_files(vec![source], &options);
});

fn split_input(input: &str) -> (&'static str, &str) {
    let Some(first) = input.chars().next() else {
        return (FORMATS[0], "");
    };
    let format = FORMATS[first as usize % FORMATS.len()];
    (format, &input[first.len_utf8()..])
}
