//! Export scan / analysis results in multiple formats.

pub mod csv;
pub mod excel;
pub mod html;
pub mod json;
pub mod pdf;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::scanner::ScanReport;
use crate::Result;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExportFormat {
    #[default]
    Pdf,
    Excel,
    Html,
    Csv,
    Json,
}

impl ExportFormat {
    pub const ALL: &'static [ExportFormat] = &[
        ExportFormat::Pdf,
        ExportFormat::Excel,
        ExportFormat::Html,
        ExportFormat::Csv,
        ExportFormat::Json,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "PDF",
            ExportFormat::Excel => "Excel (.xlsx)",
            ExportFormat::Html => "HTML",
            ExportFormat::Csv => "CSV",
            ExportFormat::Json => "JSON",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Excel => "xlsx",
            ExportFormat::Html => "html",
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
        }
    }
}

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
