use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use serde::Serialize;

use crate::tokenizer::Location;

/// Git blame lines keyed by line number.
pub type BlamedLines = BTreeMap<String, BlamedLine>;

/// Git blame information for one duplicated source line.
#[derive(Clone, Debug, Serialize)]
pub struct BlamedLine {
    /// Commit revision.
    pub rev: String,
    /// Author name reported by Git.
    pub author: String,
    /// Author or commit date reported by Git.
    pub date: String,
    /// Source line text.
    pub line: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct SourceId(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct FormatId(pub(super) usize);

/// One duplicated fragment in a source file.
#[derive(Clone, Debug, Serialize)]
pub struct Fragment {
    #[serde(rename = "sourceId")]
    /// Source identifier, usually a path.
    pub source_id: String,
    /// Start location of the duplicated fragment.
    pub start: Location,
    /// End location of the duplicated fragment.
    pub end: Location,
    /// Byte range of the duplicated fragment.
    pub range: [usize; 2],
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional Git blame information keyed by line number.
    pub blame: Option<BlamedLines>,
}

/// Pair of duplicated fragments reported as one clone.
#[derive(Clone, Debug, Serialize)]
pub struct CloneMatch {
    /// Format name shared by both fragments.
    pub format: String,
    #[serde(rename = "duplicationA")]
    /// First duplicated fragment.
    pub duplication_a: Fragment,
    #[serde(rename = "duplicationB")]
    /// Second duplicated fragment.
    pub duplication_b: Fragment,
    /// Number of detection tokens in the clone.
    pub tokens: usize,
}

/// Clone skipped from final output with compatibility/debug messages.
#[derive(Clone, Debug)]
pub struct SkippedClone {
    /// Skipped clone candidate.
    pub clone: CloneMatch,
    /// Reason messages explaining why the clone was skipped.
    pub message: Vec<String>,
}

/// Aggregated duplication counters for a source, format, or whole run.
#[derive(Clone, Debug, Default, Serialize)]
pub struct StatisticRow {
    /// Total line count.
    pub lines: usize,
    /// Total token count.
    pub tokens: usize,
    /// Number of sources included in the row.
    pub sources: usize,
    /// Number of clone pairs.
    pub clones: usize,
    #[serde(rename = "duplicatedLines")]
    /// Number of lines covered by at least one clone.
    pub duplicated_lines: usize,
    #[serde(rename = "duplicatedTokens")]
    /// Number of duplicated tokens.
    pub duplicated_tokens: usize,
    /// Duplicated line percentage.
    pub percentage: f64,
    #[serde(rename = "percentageTokens")]
    /// Duplicated token percentage.
    pub percentage_tokens: f64,
    #[serde(rename = "newDuplicatedLines")]
    /// New duplicated line count, kept for upstream report shape.
    pub new_duplicated_lines: usize,
    #[serde(rename = "newClones")]
    /// New clone count, kept for upstream report shape.
    pub new_clones: usize,
}

/// Duplication statistics grouped by format.
#[derive(Clone, Debug, Default, Serialize)]
pub struct FormatStatistic {
    /// Per-source statistics for this format.
    pub sources: HashMap<String, StatisticRow>,
    /// Total statistics for this format.
    pub total: StatisticRow,
}

/// Duplication statistics for a full detection run.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Statistics {
    /// Total statistics across all formats.
    pub total: StatisticRow,
    /// Statistics grouped by format name.
    pub formats: HashMap<String, FormatStatistic>,
}

/// Summary of one analyzed source.
#[derive(Clone, Debug, Serialize)]
pub struct SourceSummary {
    /// Source path or identifier.
    pub path: String,
    /// Detected or assigned format.
    pub format: String,
    /// Source line count.
    pub lines: usize,
    /// Detection token count.
    pub tokens: usize,
}

/// Complete detector output.
#[derive(Clone, Debug, Serialize)]
pub struct DetectionResult {
    /// Reported clone pairs.
    pub clones: Vec<CloneMatch>,
    #[serde(skip)]
    /// Clone candidates skipped from final reports.
    pub skipped_clones: Vec<SkippedClone>,
    /// Aggregate statistics.
    pub statistics: Statistics,
    /// Analyzed source summaries.
    pub sources: Vec<SourceSummary>,
    #[serde(skip)]
    /// Source contents keyed by source identifier for reporters that need
    /// fragments.
    pub source_contents: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub(super) struct TokenSpan {
    pub(super) start: Location,
    pub(super) end: Location,
    pub(super) range: [usize; 2],
}

#[derive(Clone, Debug)]
pub(super) struct SourceMeta {
    pub(super) source_id: String,
    pub(super) format: String,
    pub(super) lines: usize,
    pub(super) tokens: usize,
}

#[derive(Clone, Debug)]
pub(super) struct TokenStream {
    pub(super) source_id: SourceId,
    pub(super) format_id: FormatId,
    pub(super) hashes: Vec<u64>,
    pub(super) spans: Vec<TokenSpan>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Occurrence {
    pub(super) source_id: SourceId,
    pub(super) token_start: usize,
}

#[derive(Clone, Debug)]
pub(super) struct PreparedSource {
    pub(super) meta: SourceMeta,
    pub(super) stream: TokenStream,
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedSourceDraft {
    pub(super) meta: SourceMeta,
    pub(super) content: Arc<str>,
    pub(super) hashes: Arc<Vec<u64>>,
    pub(super) spans: Arc<Vec<TokenSpan>>,
}
