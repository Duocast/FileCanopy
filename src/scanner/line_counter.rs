use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineCountReport {
    pub per_file: Vec<FileLineCount>,
    pub total_lines: u64,
    pub monolithic: Vec<FileLineCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLineCount {
    pub path: std::path::PathBuf,
    pub lines: u64,
    pub bytes: u64,
}

/// Count lines in matching files under `root`. `extensions` are matched
/// case-insensitively against the file extension (no leading dot).
pub fn count(
    _root: &Path,
    _extensions: &[String],
    _monolith_threshold: usize,
) -> Result<LineCountReport> {
    // TODO: rayon-parallel newline count, optionally skipping binary files
    Ok(LineCountReport::default())
}
