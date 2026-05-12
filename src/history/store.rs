use std::path::Path;

use crate::history::Snapshot;
use crate::Result;

/// SQLite-backed snapshot store. Snapshots are looked up by `id` (auto-assigned
/// ULID-style) or by label.
pub struct SnapshotStore {
    // TODO: rusqlite::Connection
}

impl SnapshotStore {
    pub fn open(_path: &Path) -> Result<Self> {
        // TODO: open sqlite, run migrations
        Ok(Self {})
    }

    pub fn put(&mut self, _snapshot: &Snapshot) -> Result<()> {
        // TODO
        Ok(())
    }

    pub fn get(&self, _id_or_label: &str) -> Result<Option<Snapshot>> {
        // TODO
        Ok(None)
    }

    pub fn list(&self) -> Result<Vec<Snapshot>> {
        // TODO
        Ok(Vec::new())
    }

    pub fn delete(&mut self, _id_or_label: &str) -> Result<bool> {
        // TODO
        Ok(false)
    }
}
