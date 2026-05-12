use std::fs;
use std::path::Path;

use crate::scanner::ScanReport;
use crate::Result;

/// Pretty-printed JSON dump of a scan report. Convenient as an intermediate
/// format for piping into other tools.
pub fn export(report: &ScanReport, out: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| crate::Error::Report(e.to_string()))?;
    fs::write(out, json)?;
    Ok(())
}
