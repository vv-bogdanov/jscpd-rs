use anyhow::Result;

use super::file_output::write_file_report;
use super::summary::summary_rows;
use crate::cli::Options;
use crate::detector::Statistics;

pub(super) fn write(result: &crate::detector::DetectionResult, options: &Options) -> Result<()> {
    let csv = CsvReport::from_statistics(&result.statistics).to_string();
    write_file_report(options, "jscpd-report.csv", "CSV report", csv)
}

struct CsvReport {
    rows: Vec<[String; 7]>,
}

impl CsvReport {
    fn from_statistics(statistics: &Statistics) -> Self {
        Self {
            rows: summary_rows(statistics),
        }
    }
}

impl std::fmt::Display for CsvReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, row) in self.rows.iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", row.join(","))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::test_support::{make_test_statistics, write_test_report};

    #[test]
    fn csv_report_matches_upstream_summary_shape() {
        let stats = make_test_statistics();
        let report = CsvReport::from_statistics(&stats);
        let csv = report.to_string();

        assert_eq!(
            csv,
            [
                "Format,Files analyzed,Total lines,Total tokens,Clones found,Duplicated lines,Duplicated tokens",
                "javascript,2,20,100,1,5 (25%),30 (30%)",
                "Total:,2,20,100,1,5 (25%),30 (30%)",
            ]
            .join("\n")
        );
    }

    #[test]
    fn write_reports_writes_csv_report() {
        let csv = write_test_report("csv", "csv-report", &["jscpd-report.csv"]);

        assert!(csv.starts_with("Format,Files analyzed,Total lines"));
    }
}
