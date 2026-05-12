use std::path::Path;

use crate::scanner::ScanReport;
use crate::Result;

/// Flat CSV: one row per file with size, on-disk size, modified-time, and
/// optional line count.
pub fn export(_report: &ScanReport, _out: &Path) -> Result<()> {
    // TODO
    Ok(())
}
