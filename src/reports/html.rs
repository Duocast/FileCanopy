use std::path::Path;

use crate::scanner::ScanReport;
use crate::Result;

/// Render a self-contained HTML report (inline CSS/SVG, no external assets).
pub fn export(_report: &ScanReport, _out: &Path) -> Result<()> {
    // TODO: askama template at templates/report.html
    Ok(())
}
