use std::path::{Path, PathBuf};
use std::sync::Mutex;

use jwalk::{Parallelism, WalkDirGeneric};
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::scanner::metadata::{EntryKind, FileEntry};

#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub roots: Vec<PathBuf>,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    /// File or directory names to skip. Matched against each path component
    /// exactly (e.g. `"node_modules"`, `".git"`). Full glob support is a
    /// follow-up; this covers the common cases.
    pub excludes: Vec<String>,
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanReport {
    pub entries: Vec<FileEntry>,
    pub total_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub errors: Vec<String>,
}

/// Walk the filesystem in parallel and produce a flat list of entries.
///
/// Each root in `opts.roots` is walked independently and the results are
/// merged into a single [`ScanReport`]. Errors encountered on individual
/// entries are collected into `ScanReport::errors` rather than aborting
/// the scan.
pub fn scan(opts: &ScanOptions) -> Result<ScanReport> {
    let mut report = ScanReport::default();
    for root in &opts.roots {
        let r = scan_root(root, opts)?;
        merge(&mut report, r);
    }
    Ok(report)
}

/// Walk a single root.
pub fn scan_root(root: &Path, opts: &ScanOptions) -> Result<ScanReport> {
    let parallelism = match opts.threads {
        Some(0) | None => Parallelism::RayonDefaultPool {
            busy_timeout: std::time::Duration::from_secs(1),
        },
        Some(n) => Parallelism::RayonNewPool(n),
    };

    let excludes = opts.excludes.clone();
    let walker = WalkDirGeneric::<((), ())>::new(root)
        .follow_links(opts.follow_symlinks)
        .skip_hidden(false)
        .parallelism(parallelism)
        .process_read_dir(move |_depth, _path, _state, children| {
            if excludes.is_empty() {
                return;
            }
            children.retain(|res| match res {
                Ok(de) => {
                    let name = de.file_name().to_string_lossy();
                    !excludes.iter().any(|e| e == name.as_ref())
                }
                Err(_) => true,
            });
        });

    let walker = if let Some(d) = opts.max_depth {
        walker.max_depth(d)
    } else {
        walker
    };

    let report = Mutex::new(ScanReport::default());

    for entry in walker {
        let de = match entry {
            Ok(de) => de,
            Err(e) => {
                report.lock().unwrap().errors.push(e.to_string());
                continue;
            }
        };

        // Skip the root itself from the entry list — it appears as depth 0
        // and isn't a meaningful "file or directory inside the scan".
        if de.depth() == 0 {
            continue;
        }

        let file_type = de.file_type();
        let path = de.path();

        let kind = if file_type.is_symlink() {
            EntryKind::Symlink
        } else if file_type.is_dir() {
            EntryKind::Directory
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };

        let metadata = match de.metadata() {
            Ok(m) => m,
            Err(e) => {
                report
                    .lock()
                    .unwrap()
                    .errors
                    .push(format!("{}: {}", path.display(), e));
                continue;
            }
        };

        let size = if metadata.is_file() { metadata.len() } else { 0 };
        let size_on_disk = allocated_size(&metadata);
        let modified = metadata.modified().ok();

        let entry = FileEntry {
            path,
            size,
            size_on_disk,
            kind,
            modified,
            line_count: None,
        };

        let mut g = report.lock().unwrap();
        match entry.kind {
            EntryKind::File => {
                g.file_count += 1;
                g.total_bytes += entry.size;
            }
            EntryKind::Directory => g.dir_count += 1,
            _ => {}
        }
        g.entries.push(entry);
    }

    Ok(report.into_inner().unwrap())
}

fn merge(into: &mut ScanReport, from: ScanReport) {
    into.entries.extend(from.entries);
    into.total_bytes += from.total_bytes;
    into.file_count += from.file_count;
    into.dir_count += from.dir_count;
    into.errors.extend(from.errors);
}

#[cfg(unix)]
fn allocated_size(m: &std::fs::Metadata) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    Some(m.blocks() * 512)
}

#[cfg(windows)]
fn allocated_size(_m: &std::fs::Metadata) -> Option<u64> {
    // Reporting allocated size on Windows requires GetCompressedFileSizeW
    // per-path; deferred to platform::windows.
    None
}

#[cfg(not(any(unix, windows)))]
fn allocated_size(_m: &std::fs::Metadata) -> Option<u64> {
    None
}
