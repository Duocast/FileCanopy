use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::scanner::metadata::{EntryKind, FileEntry};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineCountReport {
    pub per_file: Vec<FileLineCount>,
    pub total_lines: u64,
    pub monolithic: Vec<FileLineCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLineCount {
    pub path: std::path::PathBuf,
    pub lines: u64,
    pub bytes: u64,
}

/// Count lines across `entries` whose extension matches `extensions`
/// (case-insensitive, no leading dot). Files with a NUL byte in their first
/// 8 KiB are treated as binary and skipped.
pub fn count_entries(
    entries: &[FileEntry],
    extensions: &[String],
    monolith_threshold: usize,
) -> Result<LineCountReport> {
    let normalized: Vec<String> = extensions
        .iter()
        .map(|e| e.trim_start_matches('.').to_ascii_lowercase())
        .collect();

    let per_file: Vec<FileLineCount> = entries
        .par_iter()
        .filter(|e| e.kind == EntryKind::File)
        .filter(|e| matches_extension(&e.path, &normalized))
        .filter_map(|e| count_file(&e.path).map(|(lines, bytes)| FileLineCount {
            path: e.path.clone(),
            lines,
            bytes,
        }))
        .collect();

    let total_lines = per_file.iter().map(|f| f.lines).sum();
    let threshold = monolith_threshold as u64;
    let mut monolithic: Vec<FileLineCount> = per_file
        .iter()
        .filter(|f| f.lines >= threshold)
        .cloned()
        .collect();
    monolithic.sort_by(|a, b| b.lines.cmp(&a.lines));

    Ok(LineCountReport {
        per_file,
        total_lines,
        monolithic,
    })
}

/// Convenience wrapper that walks `root` and counts matching files.
pub fn count(
    root: &Path,
    extensions: &[String],
    monolith_threshold: usize,
) -> Result<LineCountReport> {
    let opts = crate::scanner::ScanOptions {
        roots: vec![root.to_path_buf()],
        ..Default::default()
    };
    let report = crate::scanner::walker::scan(&opts)?;
    count_entries(&report.entries, extensions, monolith_threshold)
}

fn matches_extension(path: &Path, normalized: &[String]) -> bool {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return false;
    };
    let ext = ext.to_ascii_lowercase();
    normalized.iter().any(|e| e == &ext)
}

fn count_file(path: &Path) -> Option<(u64, u64)> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut buf = [0u8; 64 * 1024];
    let mut lines: u64 = 0;
    let mut bytes: u64 = 0;
    let mut last_byte: u8 = 0;
    let mut checked_binary = false;

    loop {
        let n = reader.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        let chunk = &buf[..n];
        if !checked_binary {
            let probe_end = chunk.len().min(8192);
            if chunk[..probe_end].contains(&0) {
                return None;
            }
            checked_binary = true;
        }
        lines += chunk.iter().filter(|&&b| b == b'\n').count() as u64;
        bytes += n as u64;
        last_byte = chunk[n - 1];
    }

    if bytes > 0 && last_byte != b'\n' {
        lines += 1;
    }

    Some((lines, bytes))
}
