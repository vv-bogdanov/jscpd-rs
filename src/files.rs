mod discovery;
mod gitignore;
mod paths;
mod shebang;

#[cfg(test)]
mod tests;

pub use discovery::discover;
pub(crate) use gitignore::collect_cwd_gitignore_patterns;

/// Source content prepared by the caller for in-memory detection.
///
/// Use this with `detect_source_files` when an integration already owns file
/// contents or wants to check generated snippets without reading from disk.
#[derive(Clone, Debug)]
pub struct SourceFile {
    /// Stable source identifier used in clone reports, usually a file path.
    pub source_id: String,
    /// Source format name, for example `javascript`, `typescript`, `rust`, or
    /// another value returned by `get_supported_formats`.
    pub format: String,
    /// Full source text.
    pub content: String,
}
