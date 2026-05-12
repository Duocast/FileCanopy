//! Apply a deduplication strategy to a duplicates report.

use serde::{Deserialize, Serialize};

use crate::analysis::DuplicatesReport;
use crate::cli::args::DedupStrategy;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DedupOutcome {
    pub kept: usize,
    pub deleted: usize,
    pub hardlinked: usize,
    pub symlinked: usize,
    pub bytes_reclaimed: u64,
    pub errors: Vec<String>,
}

pub fn apply(_report: &DuplicatesReport, _strategy: DedupStrategy) -> Result<DedupOutcome> {
    // TODO: each strategy picks one canonical file per group and replaces the rest.
    //   - DryRun: report what would happen, change nothing
    //   - Delete: unlink redundant copies
    //   - Hardlink: replace duplicates with hardlinks (same filesystem only)
    //   - Symlink: replace duplicates with symbolic links
    Ok(DedupOutcome::default())
}
