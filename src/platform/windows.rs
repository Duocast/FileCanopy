use std::path::Path;

use crate::Result;

pub struct Win;

impl super::DiskMetrics for Win {
    fn size_on_disk(_path: &Path) -> Result<Option<u64>> {
        // TODO: GetCompressedFileSizeW or GetFileInformationByHandleEx
        Ok(None)
    }

    fn hardlink_count(_path: &Path) -> Result<u64> {
        // TODO: BY_HANDLE_FILE_INFORMATION::nNumberOfLinks
        Ok(1)
    }

    fn is_mount_point(_path: &Path) -> Result<bool> {
        // TODO: GetVolumeInformationByHandleW
        Ok(false)
    }
}
