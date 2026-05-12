use std::path::Path;

use crate::scanner::ScanReport;
use crate::Result;

/// Export a multi-sheet workbook: Summary, Largest Files, Directory Sizes,
/// Duplicates, Line Counts.
pub fn export(_report: &ScanReport, _out: &Path) -> Result<()> {
    // TODO: rust_xlsxwriter::Workbook with one sheet per analysis
    Ok(())
}
