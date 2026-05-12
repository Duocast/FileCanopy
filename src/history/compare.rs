use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::history::Snapshot;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffReport {
    pub added: Vec<PathDelta>,
    pub removed: Vec<PathDelta>,
    pub grown: Vec<PathDelta>,
    pub shrunk: Vec<PathDelta>,
    pub total_delta_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathDelta {
    pub path: PathBuf,
    pub before: u64,
    pub after: u64,
    pub delta: i64,
}

/// Diff two snapshots; `min_delta` filters out small fluctuations.
pub fn diff(_before: &Snapshot, _after: &Snapshot, _min_delta: u64) -> Result<DiffReport> {
    // TODO
    Ok(DiffReport::default())
}
