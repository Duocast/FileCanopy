use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::scanner::ScanReport;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub label: Option<String>,
    pub root: PathBuf,
    pub taken_at: DateTime<Utc>,
    pub report: ScanReport,
}

/// Capture a snapshot from a fresh scan.
pub fn take(_root: PathBuf, _label: Option<String>) -> Result<Snapshot> {
    // TODO: drive scanner, build Snapshot, persist via SnapshotStore
    Err(crate::Error::History("not yet implemented".into()))
}
