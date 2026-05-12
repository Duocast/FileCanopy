use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::Result;

pub struct Posix;

impl super::DiskMetrics for Posix {
    fn size_on_disk(path: &Path) -> Result<Option<u64>> {
        let m = std::fs::symlink_metadata(path)?;
        Ok(Some(m.blocks() * 512))
    }

    fn hardlink_count(path: &Path) -> Result<u64> {
        let m = std::fs::symlink_metadata(path)?;
        Ok(m.nlink())
    }

    fn is_mount_point(path: &Path) -> Result<bool> {
        let m = std::fs::symlink_metadata(path)?;
        let parent = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            // Filesystem root has no parent and is by definition a mount point.
            _ => return Ok(true),
        };
        let parent_m = std::fs::symlink_metadata(parent)?;
        Ok(m.dev() != parent_m.dev())
    }
}
