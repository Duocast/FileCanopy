use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::scanner::metadata::FileEntry;

#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub roots: Vec<PathBuf>,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    pub excludes: Vec<String>,
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanReport {
    pub entries: Vec<FileEntry>,
    pub total_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub errors: Vec<String>,
}

/// Walk the filesystem in parallel and produce a flat list of entries.
pub fn scan(_opts: &ScanOptions) -> Result<ScanReport> {
    // TODO: jwalk parallel traversal + rayon for metadata fan-out
    Ok(ScanReport::default())
}

/// Walk a single root; useful for tests.
pub fn scan_root(_root: &Path, _opts: &ScanOptions) -> Result<ScanReport> {
    // TODO
    Ok(ScanReport::default())
}
