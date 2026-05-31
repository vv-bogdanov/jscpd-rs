use crate::detector::{DetectionResult, StatisticRow, Statistics};

pub(super) fn summary_rows(statistics: &Statistics) -> Vec<[String; 7]> {
    let mut rows = vec![summary_header_row()];
    let mut formats = statistics.formats.iter().collect::<Vec<_>>();
    formats.sort_by_key(|(format, _)| *format);
    for (format, statistic) in formats {
        rows.push(statistic_to_summary_row(format, &statistic.total));
    }
    rows.push(statistic_to_summary_row("Total:", &statistics.total));
    rows
}

fn summary_header_row() -> [String; 7] {
    [
        "Format".to_string(),
        "Files analyzed".to_string(),
        "Total lines".to_string(),
        "Total tokens".to_string(),
        "Clones found".to_string(),
        "Duplicated lines".to_string(),
        "Duplicated tokens".to_string(),
    ]
}

pub(super) fn statistic_to_summary_row(format: &str, statistic: &StatisticRow) -> [String; 7] {
    [
        format.to_string(),
        statistic.sources.to_string(),
        statistic.lines.to_string(),
        statistic.tokens.to_string(),
        statistic.clones.to_string(),
        format!("{} ({}%)", statistic.duplicated_lines, statistic.percentage),
        format!(
            "{} ({}%)",
            statistic.duplicated_tokens, statistic.percentage_tokens
        ),
    ]
}

pub(super) fn silent_summary(result: &DetectionResult) -> String {
    format!(
        "Duplications detection: Found {} exact clones with {}({}%) duplicated lines in {} ({} formats) files.",
        result.clones.len(),
        result.statistics.total.duplicated_lines,
        result.statistics.total.percentage,
        result.statistics.total.sources,
        result.statistics.formats.len(),
    )
}
