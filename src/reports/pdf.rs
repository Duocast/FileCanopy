use std::path::Path;

use crate::scanner::ScanReport;
use crate::Result;

/// Render a PDF report including summary stats, top-N files, and an embedded
/// treemap snapshot.
pub fn export(_report: &ScanReport, _out: &Path) -> Result<()> {
    // TODO: build with `printpdf`; embed treemap PNG produced by visualization::treemap
    Ok(())
}
