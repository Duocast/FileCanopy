use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    /// Allocation/cluster size on disk; may differ from logical size on Windows.
    pub size_on_disk: Option<u64>,
    pub kind: EntryKind,
    pub modified: Option<SystemTime>,
    pub line_count: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl FileEntry {
    pub fn is_file(&self) -> bool {
        matches!(self.kind, EntryKind::File)
    }
}
