use crate::scanner::FileEntry;
use crate::scanner::ScanReport;

/// Return the `n` largest files, sorted descending by size.
pub fn top(report: &ScanReport, n: usize, min_size: Option<u64>) -> Vec<FileEntry> {
    let mut files: Vec<FileEntry> = report
        .entries
        .iter()
        .filter(|e| e.is_file())
        .filter(|e| min_size.is_none_or(|m| e.size >= m))
        .cloned()
        .collect();
    files.sort_by(|a, b| b.size.cmp(&a.size));
    files.truncate(n);
    files
}
