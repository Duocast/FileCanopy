//! Cross-platform abstractions for filesystem queries that differ between
//! Windows and POSIX (allocated size, hardlink count, mount-point detection).

#[cfg(unix)]
mod linux;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use linux::*;
#[cfg(windows)]
pub use windows::*;

use std::path::Path;

use crate::Result;

/// Cluster/allocation size on disk, if the OS can report it.
pub trait DiskMetrics {
    fn size_on_disk(path: &Path) -> Result<Option<u64>>;
    fn hardlink_count(path: &Path) -> Result<u64>;
    fn is_mount_point(path: &Path) -> Result<bool>;
}
