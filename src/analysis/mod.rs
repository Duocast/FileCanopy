//! Aggregation, ranking, and duplicate detection over scanned entries.

pub mod duplicates;
pub mod hasher;
pub mod largest;
pub mod tree;

pub use duplicates::{DuplicateGroup, DuplicatesReport};
pub use tree::{DirNode, SizeTree};
