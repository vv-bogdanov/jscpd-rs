use anyhow::Result;

use super::file_output::write_file_report;
use super::summary::{silent_summary, summary_rows};
use crate::cli::Options;
use crate::detector::DetectionResult;

pub(super) fn write(result: &DetectionResult, options: &Options) -> Result<()> {
    let md = MarkdownReport::from_detection(result).to_string();
    write_file_report(options, "jscpd-report.md", "Markdown report", md)
}

struct MarkdownReport {
    summary_line: String,
    rows: Vec<[String; 7]>,
}

impl MarkdownReport {
    fn from_detection(result: &DetectionResult) -> Self {
        let stats = &result.statistics;
        let summary_line = format!("> {}", silent_summary(result));
        let mut rows = summary_rows(stats);
        if let Some(total) = rows.last_mut() {
            for cell in total {
                *cell = format!("**{cell}**");
            }
        }

        Self { summary_line, rows }
    }
}

impl std::fmt::Display for MarkdownReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "# Copy/paste detection report")?;
        writeln!(f)?;
        writeln!(f, "{}", self.summary_line)?;
        writeln!(f)?;
        let widths = markdown_column_widths(&self.rows);
        for (row_idx, row) in self.rows.iter().enumerate() {
            write_markdown_row(f, row, &widths)?;
            if row_idx == 0 {
                write_markdown_separator(f, &widths)?;
            }
        }
        Ok(())
    }
}

fn markdown_column_widths(rows: &[[String; 7]]) -> [usize; 7] {
    let mut widths = [0usize; 7];
    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(cell.len());
        }
    }
    widths
}

fn write_markdown_row(
    f: &mut std::fmt::Formatter<'_>,
    row: &[String; 7],
    widths: &[usize; 7],
) -> std::fmt::Result {
    write!(f, "|")?;
    for (idx, cell) in row.iter().enumerate() {
        write!(f, " {cell:<width$} |", width = widths[idx])?;
    }
    writeln!(f)
}

fn write_markdown_separator(
    f: &mut std::fmt::Formatter<'_>,
    widths: &[usize; 7],
) -> std::fmt::Result {
    write!(f, "|")?;
    for width in widths {
        write!(f, " {:-<width$} |", "", width = *width)?;
    }
    writeln!(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::test_support::write_test_report;

    #[test]
    fn write_reports_writes_markdown_report() {
        let md = write_test_report("markdown", "markdown-report", &["jscpd-report.md"]);

        assert!(md.starts_with("\n# Copy/paste detection report"));
        assert!(md.contains("> Duplications detection:"));
    }

    #[test]
    fn markdown_report_matches_upstream_summary_shape() {
        let result = crate::detector::DetectionResult {
            clones: Vec::new(),
            skipped_clones: Vec::new(),
            statistics: crate::report::test_support::make_test_statistics(),
            sources: Vec::new(),
            source_contents: std::collections::HashMap::new(),
        };
        let md = MarkdownReport::from_detection(&result).to_string();

        assert_eq!(
            md,
            [
                "",
                "# Copy/paste detection report",
                "",
                "> Duplications detection: Found 0 exact clones with 5(25%) duplicated lines in 2 (1 formats) files.",
                "",
                "| Format     | Files analyzed | Total lines | Total tokens | Clones found | Duplicated lines | Duplicated tokens |",
                "| ---------- | -------------- | ----------- | ------------ | ------------ | ---------------- | ----------------- |",
                "| javascript | 2              | 20          | 100          | 1            | 5 (25%)          | 30 (30%)          |",
                "| **Total:** | **2**          | **20**      | **100**      | **1**        | **5 (25%)**      | **30 (30%)**      |",
                "",
            ]
            .join("\n")
        );
    }
}
