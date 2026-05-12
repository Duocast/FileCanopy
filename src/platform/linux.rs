use std::path::Path;

use crate::Result;

pub struct Posix;

impl super::DiskMetrics for Posix {
    fn size_on_disk(_path: &Path) -> Result<Option<u64>> {
        // TODO: stat(2) `st_blocks * 512`
        Ok(None)
    }

    fn hardlink_count(_path: &Path) -> Result<u64> {
        // TODO: stat(2) `st_nlink`
        Ok(1)
    }

    fn is_mount_point(_path: &Path) -> Result<bool> {
        // TODO: compare st_dev to parent's
        Ok(false)
    }
}
