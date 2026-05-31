use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::cli::Options;

pub(super) fn ensure_output_dir(options: &Options) -> Result<()> {
    fs::create_dir_all(&options.output)
        .with_context(|| format!("failed to create output dir `{}`", options.output.display()))
}

pub(super) fn write_path(path: &Path, saved_prefix: &str, content: impl AsRef<[u8]>) -> Result<()> {
    fs::write(path, content).with_context(|| format!("failed to write `{}`", path.display()))?;
    println!("{saved_prefix} saved to {}", path.display());
    Ok(())
}

pub(super) fn write_file_report(
    options: &Options,
    file_name: &str,
    saved_prefix: &str,
    content: impl AsRef<[u8]>,
) -> Result<()> {
    ensure_output_dir(options)?;
    let path = options.output.join(file_name);
    write_path(&path, saved_prefix, content)
}
