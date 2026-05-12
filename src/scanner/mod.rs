//! Filesystem traversal and per-file metadata collection.

pub mod line_counter;
pub mod metadata;
pub mod walker;

pub use metadata::FileEntry;
pub use walker::{ScanOptions, ScanReport};
