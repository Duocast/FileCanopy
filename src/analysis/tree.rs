use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::scanner::ScanReport;
use crate::Result;

/// A hierarchical view of disk usage, suitable for treemap rendering or
/// drill-down navigation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizeTree {
    pub root: DirNode,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirNode {
    pub path: PathBuf,
    pub size: u64,
    pub file_count: u64,
    pub children: Vec<DirNode>,
    pub files: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: PathBuf,
    pub size: u64,
}

/// Build a hierarchical tree from a flat scan report.
pub fn build(_report: &ScanReport) -> Result<SizeTree> {
    // TODO: fold paths into a trie keyed on path components
    Ok(SizeTree::default())
}
