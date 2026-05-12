use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::analysis::hasher::{self, Fingerprint, HashAlgo};
use crate::scanner::ScanReport;
use crate::Result;

/// Bytes hashed in the cheap prefix pre-filter. Files whose first
/// `PREFIX_BYTES` differ cannot be duplicates, so we skip the much more
/// expensive full-content hash on the survivors of the size filter.
const PREFIX_BYTES: u64 = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DuplicatesReport {
    pub groups: Vec<DuplicateGroup>,
    /// Total bytes that could be reclaimed by deduplicating (keeps one copy).
    pub reclaimable_bytes: u64,
    /// Algorithm used to derive the per-group `fingerprint`.
    pub algo: Option<HashAlgo>,
    /// Paths skipped due to read errors during hashing.
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub fingerprint: Fingerprint,
    pub size: u64,
    pub paths: Vec<PathBuf>,
}

/// Strategy: group by size → prefix-hash → full-content hash. Files of unique
/// size are skipped without ever being hashed. Files smaller than `min_size`
/// and zero-byte files are excluded entirely.
pub fn find(
    report: &ScanReport,
    min_size: u64,
    algo: HashAlgo,
) -> Result<DuplicatesReport> {
    // 1. Bucket eligible files by size.
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for entry in &report.entries {
        if !entry.is_file() {
            continue;
        }
        if entry.size == 0 || entry.size < min_size {
            continue;
        }
        by_size
            .entry(entry.size)
            .or_default()
            .push(entry.path.clone());
    }

    let mut out = DuplicatesReport {
        algo: Some(algo),
        ..DuplicatesReport::default()
    };

    for (size, paths) in by_size {
        if paths.len() < 2 {
            continue;
        }

        // 2. Within a size-bucket, group by prefix hash.
        let mut by_prefix: HashMap<Fingerprint, Vec<PathBuf>> = HashMap::new();
        for p in paths {
            match hasher::hash_prefix(&p, algo, PREFIX_BYTES) {
                Ok(fp) => by_prefix.entry(fp).or_default().push(p),
                Err(e) => out.errors.push(format!("{}: {}", p.display(), e)),
            }
        }

        // 3. Within a prefix-bucket of >1 candidates, derive the full-content
        //    hash and emit groups of identical content.
        for (_prefix, candidates) in by_prefix {
            if candidates.len() < 2 {
                continue;
            }
            let mut by_full: HashMap<Fingerprint, Vec<PathBuf>> = HashMap::new();
            for p in candidates {
                match hasher::hash_file(&p, algo) {
                    Ok(fp) => by_full.entry(fp).or_default().push(p),
                    Err(e) => out.errors.push(format!("{}: {}", p.display(), e)),
                }
            }
            for (fp, mut group_paths) in by_full {
                if group_paths.len() < 2 {
                    continue;
                }
                // Refuse to emit a group with an unset/sentinel fingerprint —
                // it would defeat downstream safety checks in dedup::apply.
                if fp.is_empty() {
                    out.errors.push(format!(
                        "skipped {} same-content candidates: empty fingerprint (size {})",
                        group_paths.len(),
                        size,
                    ));
                    continue;
                }
                group_paths.sort();
                let n = group_paths.len() as u64;
                out.reclaimable_bytes += size.saturating_mul(n - 1);
                out.groups.push(DuplicateGroup {
                    fingerprint: fp,
                    size,
                    paths: group_paths,
                });
            }
        }
    }

    // Stable ordering: largest-savings groups first, then by fingerprint.
    out.groups.sort_by(|a, b| {
        let a_save = a.size * (a.paths.len() as u64 - 1);
        let b_save = b.size * (b.paths.len() as u64 - 1);
        b_save
            .cmp(&a_save)
            .then_with(|| a.fingerprint.cmp(&b.fingerprint))
    });

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::metadata::{EntryKind, FileEntry};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_file(dir: &std::path::Path, name: &str, bytes: &[u8]) -> PathBuf {
        let p = dir.join(name);
        File::create(&p).unwrap().write_all(bytes).unwrap();
        p
    }

    fn entry(path: PathBuf, size: u64) -> FileEntry {
        FileEntry {
            path,
            size,
            size_on_disk: None,
            kind: EntryKind::File,
            modified: None,
            line_count: None,
        }
    }

    fn report_from(entries: Vec<FileEntry>) -> ScanReport {
        let total: u64 = entries.iter().map(|e| e.size).sum();
        ScanReport {
            entries,
            total_bytes: total,
            file_count: 0,
            dir_count: 0,
            errors: vec![],
        }
    }

    #[test]
    fn same_size_different_content_is_not_a_duplicate() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", &vec![0u8; 8192]);
        let b = write_file(tmp.path(), "b.bin", &vec![1u8; 8192]);
        let report = report_from(vec![entry(a, 8192), entry(b, 8192)]);
        let dups = find(&report, 1024, HashAlgo::Blake3).unwrap();
        assert!(
            dups.groups.is_empty(),
            "must not group same-size-different-content as duplicates: {:?}",
            dups.groups
        );
        assert_eq!(dups.reclaimable_bytes, 0);
    }

    #[test]
    fn identical_files_are_grouped() {
        let tmp = tempdir().unwrap();
        let body = b"the quick brown fox jumps over the lazy dog".repeat(64);
        let a = write_file(tmp.path(), "a.bin", &body);
        let b = write_file(tmp.path(), "b.bin", &body);
        let c = write_file(tmp.path(), "c.bin", &body);
        let size = body.len() as u64;
        let report = report_from(vec![
            entry(a.clone(), size),
            entry(b.clone(), size),
            entry(c.clone(), size),
        ]);
        let dups = find(&report, 64, HashAlgo::Blake3).unwrap();
        assert_eq!(dups.groups.len(), 1);
        assert_eq!(dups.groups[0].paths.len(), 3);
        assert_eq!(dups.reclaimable_bytes, size * 2);
        assert!(!dups.groups[0].fingerprint.is_empty());
    }

    #[test]
    fn empty_files_are_skipped() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", b"");
        let b = write_file(tmp.path(), "b.bin", b"");
        let report = report_from(vec![entry(a, 0), entry(b, 0)]);
        let dups = find(&report, 0, HashAlgo::Blake3).unwrap();
        assert!(dups.groups.is_empty());
    }

    #[test]
    fn min_size_excludes_small_files() {
        let tmp = tempdir().unwrap();
        let small = vec![7u8; 100];
        let a = write_file(tmp.path(), "a.bin", &small);
        let b = write_file(tmp.path(), "b.bin", &small);
        let report = report_from(vec![entry(a, 100), entry(b, 100)]);
        let dups = find(&report, 1024, HashAlgo::Blake3).unwrap();
        assert!(dups.groups.is_empty());
    }

    #[test]
    fn shared_prefix_distinct_tails_not_grouped() {
        let tmp = tempdir().unwrap();
        let mut a_data = vec![0u8; 4096];
        a_data.extend_from_slice(&[1u8; 4096]);
        let mut b_data = vec![0u8; 4096];
        b_data.extend_from_slice(&[2u8; 4096]);
        let a = write_file(tmp.path(), "a.bin", &a_data);
        let b = write_file(tmp.path(), "b.bin", &b_data);
        let report = report_from(vec![entry(a, 8192), entry(b, 8192)]);
        let dups = find(&report, 1024, HashAlgo::Blake3).unwrap();
        assert!(dups.groups.is_empty());
    }
}
