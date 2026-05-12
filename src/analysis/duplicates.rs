use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::analysis::hasher::Fingerprint;
use crate::cli::args::HashAlgo;
use crate::scanner::ScanReport;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DuplicatesReport {
    pub groups: Vec<DuplicateGroup>,
    /// Total bytes that could be reclaimed by deduplicating (keeps one copy).
    pub reclaimable_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub fingerprint: Fingerprint,
    pub size: u64,
    pub paths: Vec<PathBuf>,
}

/// Strategy: group by size → prefix-hash → full-content hash. Files of unique
/// size are skipped without ever being hashed.
pub fn find(
    _report: &ScanReport,
    _min_size: u64,
    _algo: HashAlgo,
) -> Result<DuplicatesReport> {
    // TODO
    Ok(DuplicatesReport::default())
}
