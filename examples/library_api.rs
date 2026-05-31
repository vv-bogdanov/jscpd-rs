use jscpd_rs::{SourceFile, detect_source_files, get_default_options};

fn main() {
    let mut options = get_default_options();
    options.reporters.clear();
    options.silent = true;
    options.no_tips = true;
    options.min_lines = 2;
    options.min_tokens = 5;

    let files = vec![
        SourceFile {
            source_id: "a.js".to_string(),
            format: "javascript".to_string(),
            content: duplicate_body(),
        },
        SourceFile {
            source_id: "b.js".to_string(),
            format: "javascript".to_string(),
            content: duplicate_body(),
        },
    ];

    let result = detect_source_files(files, &options);
    println!(
        "{} clones, {} duplicated lines, {:.2}% duplicated",
        result.clones.len(),
        result.statistics.total.duplicated_lines,
        result.statistics.total.percentage
    );
}

fn duplicate_body() -> String {
    [
        "const alpha = 1;",
        "const beta = 2;",
        "const gamma = alpha + beta;",
        "console.log(gamma);",
        "",
    ]
    .join("\n")
}
