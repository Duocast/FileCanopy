//! Persistent snapshots and over-time comparisons.

pub mod compare;
pub mod snapshot;
pub mod store;

pub use compare::DiffReport;
pub use snapshot::Snapshot;
pub use store::SnapshotStore;
