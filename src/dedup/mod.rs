//! Apply a deduplication strategy to a duplicates report.
//!
//! Every destructive strategy runs the same safety gate before touching the
//! filesystem:
//!
//! 1. The group must have a non-empty (non-sentinel) fingerprint and at least
//!    two paths.
//! 2. Every path is re-stat'd; sizes must match the recorded group size and
//!    each other.
//! 3. The canonical file and every candidate are re-hashed with Blake3 *at
//!    apply time* and must match byte-for-byte (cryptographically). The
//!    fingerprint recorded in the report is treated as an identifier only —
//!    never as proof of equality.
//!
//! Hardlink and symlink replacements use a rename-then-link pattern so the
//! original duplicate is preserved on disk until the replacement is in place,
//! and is restored if the link operation fails.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::analysis::hasher::{self, HashAlgo};
use crate::analysis::{DuplicateGroup, DuplicatesReport};
use crate::Result;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DedupStrategy {
    DryRun,
    Delete,
    Hardlink,
    Symlink,
}

impl Default for DedupStrategy {
    fn default() -> Self {
        Self::DryRun
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DedupOutcome {
    pub kept: usize,
    pub deleted: usize,
    pub hardlinked: usize,
    pub symlinked: usize,
    /// Paths the dry-run *would* have acted on if the strategy were destructive.
    pub would_act: usize,
    pub bytes_reclaimed: u64,
    pub errors: Vec<String>,
}

pub fn apply(report: &DuplicatesReport, strategy: DedupStrategy) -> Result<DedupOutcome> {
    let mut out = DedupOutcome::default();
    for group in &report.groups {
        apply_group(group, strategy, &mut out);
    }
    Ok(out)
}

fn apply_group(group: &DuplicateGroup, strategy: DedupStrategy, out: &mut DedupOutcome) {
    if group.fingerprint.is_empty() {
        out.errors.push(format!(
            "skipped group of {} files (size {}): empty fingerprint",
            group.paths.len(),
            group.size,
        ));
        return;
    }
    if group.paths.len() < 2 {
        out.errors.push(format!(
            "skipped group with {} path(s): need at least 2",
            group.paths.len(),
        ));
        return;
    }

    // Canonical = first path (paths are sorted by `duplicates::find`).
    let canonical = &group.paths[0];

    // Verify canonical metadata + hash once per group.
    let canonical_meta = match std::fs::symlink_metadata(canonical) {
        Ok(m) => m,
        Err(e) => {
            out.errors
                .push(format!("canonical {}: {}", canonical.display(), e));
            return;
        }
    };
    if !canonical_meta.file_type().is_file() {
        out.errors
            .push(format!("canonical {} is not a regular file", canonical.display()));
        return;
    }
    if canonical_meta.len() != group.size {
        out.errors.push(format!(
            "canonical {} size changed since scan ({} → {})",
            canonical.display(),
            group.size,
            canonical_meta.len(),
        ));
        return;
    }
    let canonical_hash = match hasher::hash_file(canonical, HashAlgo::Blake3) {
        Ok(h) => h,
        Err(e) => {
            out.errors
                .push(format!("re-hash canonical {}: {}", canonical.display(), e));
            return;
        }
    };

    out.kept += 1;

    for dup in &group.paths[1..] {
        match verify_and_act(canonical, &canonical_hash, group.size, dup, strategy) {
            Ok(action) => match action {
                Action::Deleted => {
                    out.deleted += 1;
                    out.bytes_reclaimed += group.size;
                }
                Action::Hardlinked => {
                    out.hardlinked += 1;
                    out.bytes_reclaimed += group.size;
                }
                Action::Symlinked => {
                    out.symlinked += 1;
                    out.bytes_reclaimed += group.size;
                }
                Action::DryRun => {
                    out.would_act += 1;
                    out.bytes_reclaimed += group.size;
                }
            },
            Err(e) => out.errors.push(e),
        }
    }
}

enum Action {
    Deleted,
    Hardlinked,
    Symlinked,
    DryRun,
}

fn verify_and_act(
    canonical: &Path,
    canonical_hash: &str,
    expected_size: u64,
    dup: &Path,
    strategy: DedupStrategy,
) -> std::result::Result<Action, String> {
    if dup == canonical {
        return Err(format!(
            "{}: candidate is the canonical path",
            dup.display()
        ));
    }

    let meta = std::fs::symlink_metadata(dup)
        .map_err(|e| format!("{}: {}", dup.display(), e))?;
    if !meta.file_type().is_file() {
        return Err(format!("{}: not a regular file", dup.display()));
    }
    if meta.len() != expected_size {
        return Err(format!(
            "{}: size changed since scan ({} → {})",
            dup.display(),
            expected_size,
            meta.len(),
        ));
    }

    // Refuse to act if canonical and dup are already the same inode (already
    // hardlinked) — there's nothing to reclaim and a Delete would actually
    // remove the only-named link to that inode here, but on hardlinked
    // siblings it would still destroy the user's reference.
    #[cfg(unix)]
    if same_inode(canonical, dup).unwrap_or(false) {
        return Err(format!(
            "{} and {} are already the same inode",
            canonical.display(),
            dup.display(),
        ));
    }

    let dup_hash = hasher::hash_file(dup, HashAlgo::Blake3)
        .map_err(|e| format!("re-hash {}: {}", dup.display(), e))?;
    if dup_hash != canonical_hash {
        return Err(format!(
            "{}: content differs from canonical at apply time — refusing to act",
            dup.display(),
        ));
    }

    match strategy {
        DedupStrategy::DryRun => Ok(Action::DryRun),
        DedupStrategy::Delete => {
            std::fs::remove_file(dup).map_err(|e| format!("delete {}: {}", dup.display(), e))?;
            Ok(Action::Deleted)
        }
        DedupStrategy::Hardlink => {
            #[cfg(unix)]
            {
                if !same_device(canonical, dup).unwrap_or(false) {
                    return Err(format!(
                        "{}: cross-filesystem hardlink to {} not allowed",
                        dup.display(),
                        canonical.display(),
                    ));
                }
            }
            replace_with(dup, |dest| std::fs::hard_link(canonical, dest))
                .map_err(|e| format!("hardlink {} → {}: {}", dup.display(), canonical.display(), e))?;
            Ok(Action::Hardlinked)
        }
        DedupStrategy::Symlink => {
            replace_with(dup, |dest| symlink_file(canonical, dest))
                .map_err(|e| format!("symlink {} → {}: {}", dup.display(), canonical.display(), e))?;
            Ok(Action::Symlinked)
        }
    }
}

/// Atomically replace `dest` by first renaming it to a temp sibling,
/// then running `link_fn(dest)`. On failure, restore the original.
fn replace_with<F>(dest: &Path, link_fn: F) -> std::io::Result<()>
where
    F: FnOnce(&Path) -> std::io::Result<()>,
{
    let tmp = tmp_sibling(dest);
    if tmp.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("temp path {} already exists; aborting", tmp.display()),
        ));
    }
    std::fs::rename(dest, &tmp)?;
    match link_fn(dest) {
        Ok(()) => {
            // Best-effort cleanup of the displaced original.
            std::fs::remove_file(&tmp)
        }
        Err(e) => {
            // Roll back: restore the original at `dest`. Ignore the rollback
            // error and surface the primary failure — the original is at
            // least still on disk under `tmp` if rollback fails.
            let _ = std::fs::rename(&tmp, dest);
            Err(e)
        }
    }
}

fn tmp_sibling(p: &Path) -> PathBuf {
    let mut s = OsString::from(p.as_os_str());
    s.push(format!(".fc_dedup_tmp.{}", std::process::id()));
    PathBuf::from(s)
}

#[cfg(unix)]
fn same_device(a: &Path, b: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::MetadataExt;
    Ok(std::fs::metadata(a)?.dev() == std::fs::metadata(b)?.dev())
}

#[cfg(unix)]
fn same_inode(a: &Path, b: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::MetadataExt;
    let am = std::fs::metadata(a)?;
    let bm = std::fs::metadata(b)?;
    Ok(am.dev() == bm.dev() && am.ino() == bm.ino())
}

#[cfg(unix)]
fn symlink_file(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn symlink_file(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(not(any(unix, windows)))]
fn symlink_file(_target: &Path, _link: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "symlink not supported on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::DuplicatesReport;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let p = dir.join(name);
        File::create(&p).unwrap().write_all(bytes).unwrap();
        p
    }

    fn group(fingerprint: &str, size: u64, paths: Vec<PathBuf>) -> DuplicateGroup {
        DuplicateGroup {
            fingerprint: fingerprint.into(),
            size,
            paths,
        }
    }

    fn report_of(groups: Vec<DuplicateGroup>) -> DuplicatesReport {
        let reclaimable: u64 = groups
            .iter()
            .map(|g| g.size * (g.paths.len() as u64 - 1))
            .sum();
        DuplicatesReport {
            groups,
            reclaimable_bytes: reclaimable,
            algo: Some(HashAlgo::Blake3),
            errors: vec![],
        }
    }

    fn fp_of(p: &Path) -> String {
        hasher::hash_file(p, HashAlgo::Blake3).unwrap()
    }

    #[test]
    fn empty_fingerprint_is_refused() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", b"data");
        let b = write_file(tmp.path(), "b.bin", b"data");
        let report = report_of(vec![group("", 4, vec![a.clone(), b.clone()])]);

        let outcome = apply(&report, DedupStrategy::Delete).unwrap();
        assert_eq!(outcome.deleted, 0);
        assert!(!outcome.errors.is_empty());
        assert!(a.exists() && b.exists(), "files must be untouched");
    }

    #[test]
    fn dry_run_does_not_touch_files() {
        let tmp = tempdir().unwrap();
        let body = b"hello hello hello".repeat(100);
        let a = write_file(tmp.path(), "a.bin", &body);
        let b = write_file(tmp.path(), "b.bin", &body);
        let size = body.len() as u64;
        let report = report_of(vec![group(&fp_of(&a), size, vec![a.clone(), b.clone()])]);

        let outcome = apply(&report, DedupStrategy::DryRun).unwrap();
        assert_eq!(outcome.kept, 1);
        assert_eq!(outcome.would_act, 1);
        assert_eq!(outcome.deleted, 0);
        assert_eq!(outcome.bytes_reclaimed, size);
        assert!(a.exists() && b.exists());
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    }

    #[test]
    fn delete_removes_duplicate_and_keeps_canonical() {
        let tmp = tempdir().unwrap();
        let body = b"duplicate me".repeat(200);
        let a = write_file(tmp.path(), "a.bin", &body);
        let b = write_file(tmp.path(), "b.bin", &body);
        let c = write_file(tmp.path(), "c.bin", &body);
        let size = body.len() as u64;
        let report = report_of(vec![group(
            &fp_of(&a),
            size,
            vec![a.clone(), b.clone(), c.clone()],
        )]);

        let outcome = apply(&report, DedupStrategy::Delete).unwrap();
        assert_eq!(outcome.kept, 1);
        assert_eq!(outcome.deleted, 2);
        assert_eq!(outcome.bytes_reclaimed, size * 2);
        assert!(a.exists(), "canonical must remain");
        assert!(!b.exists() && !c.exists(), "duplicates must be removed");
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    }

    #[test]
    fn delete_refuses_when_content_differs_at_apply_time() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", &vec![0u8; 4096]);
        let b = write_file(tmp.path(), "b.bin", &vec![1u8; 4096]);
        // Build a malicious report claiming a and b are duplicates with a's
        // fingerprint. This is exactly the failure mode the safety gate is
        // designed to catch.
        let report = report_of(vec![group(&fp_of(&a), 4096, vec![a.clone(), b.clone()])]);

        let outcome = apply(&report, DedupStrategy::Delete).unwrap();
        assert_eq!(outcome.deleted, 0);
        assert_eq!(outcome.bytes_reclaimed, 0);
        assert!(!outcome.errors.is_empty(), "safety gate must record an error");
        assert!(a.exists() && b.exists(), "neither file may be deleted");
    }

    #[test]
    fn delete_refuses_when_size_changed_since_scan() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", b"same content");
        let b = write_file(tmp.path(), "b.bin", b"same content");
        // Recorded size is wrong (file was 12 bytes; report claims 64).
        let report = report_of(vec![group(&fp_of(&a), 64, vec![a.clone(), b.clone()])]);

        let outcome = apply(&report, DedupStrategy::Delete).unwrap();
        assert_eq!(outcome.deleted, 0);
        assert!(a.exists() && b.exists());
        assert!(!outcome.errors.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn hardlink_replaces_duplicate_with_shared_inode() {
        use std::os::unix::fs::MetadataExt;
        let tmp = tempdir().unwrap();
        let body = b"link me".repeat(100);
        let a = write_file(tmp.path(), "a.bin", &body);
        let b = write_file(tmp.path(), "b.bin", &body);
        let size = body.len() as u64;
        let report = report_of(vec![group(&fp_of(&a), size, vec![a.clone(), b.clone()])]);

        let outcome = apply(&report, DedupStrategy::Hardlink).unwrap();
        assert_eq!(outcome.hardlinked, 1, "{:?}", outcome.errors);
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        assert!(a.exists() && b.exists(), "both paths must still exist");

        let ma = std::fs::metadata(&a).unwrap();
        let mb = std::fs::metadata(&b).unwrap();
        assert_eq!(ma.ino(), mb.ino(), "b should now point at a's inode");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_replaces_duplicate() {
        let tmp = tempdir().unwrap();
        let body = b"symlink me".repeat(50);
        let a = write_file(tmp.path(), "a.bin", &body);
        let b = write_file(tmp.path(), "b.bin", &body);
        let size = body.len() as u64;
        let report = report_of(vec![group(&fp_of(&a), size, vec![a.clone(), b.clone()])]);

        let outcome = apply(&report, DedupStrategy::Symlink).unwrap();
        assert_eq!(outcome.symlinked, 1, "{:?}", outcome.errors);
        let bmeta = std::fs::symlink_metadata(&b).unwrap();
        assert!(bmeta.file_type().is_symlink());
        assert_eq!(std::fs::read_link(&b).unwrap(), a);
    }

    #[test]
    fn single_path_group_is_refused() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", b"x");
        let report = report_of(vec![group(&fp_of(&a), 1, vec![a.clone()])]);
        let outcome = apply(&report, DedupStrategy::Delete).unwrap();
        assert_eq!(outcome.deleted, 0);
        assert!(!outcome.errors.is_empty());
        assert!(a.exists());
    }
}
