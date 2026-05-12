//! Export scan / analysis results in multiple formats.

pub mod csv;
pub mod excel;
pub mod html;
pub mod json;
pub mod pdf;

use std::path::Path;

use crate::cli::args::ExportFormat;
use crate::scanner::ScanReport;
use crate::Result;

/// Convenience dispatcher that picks an exporter based on `format` (or the
/// extension of `out` when `format` is `None`).
pub fn export(report: &ScanReport, out: &Path, format: Option<ExportFormat>) -> Result<()> {
    let format = format.unwrap_or_else(|| infer_format(out));
    match format {
        ExportFormat::Pdf => pdf::export(report, out),
        ExportFormat::Excel => excel::export(report, out),
        ExportFormat::Html => html::export(report, out),
        ExportFormat::Csv => csv::export(report, out),
        ExportFormat::Json => json::export(report, out),
    }
}

fn infer_format(out: &Path) -> ExportFormat {
    match out.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ext) => match ext.as_str() {
            "pdf" => ExportFormat::Pdf,
            "xlsx" | "xls" => ExportFormat::Excel,
            "html" | "htm" => ExportFormat::Html,
            "csv" => ExportFormat::Csv,
            _ => ExportFormat::Json,
        },
        None => ExportFormat::Json,
    }
}
