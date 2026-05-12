use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::Result;

/// Stable content fingerprint, lowercase hex-encoded. Widths depend on the
/// chosen [`HashAlgo`] (256-bit for Blake3, 128-bit for xxh3). Comparisons
/// are only meaningful between fingerprints produced by the same algorithm.
pub type Fingerprint = String;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HashAlgo {
    Blake3,
    Xxh3,
}

impl Default for HashAlgo {
    fn default() -> Self {
        Self::Blake3
    }
}

const READ_BUF: usize = 64 * 1024;

fn io_err(path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        path: Some(path.to_path_buf()),
        source,
    }
}

/// Hash a file's full contents with the chosen algorithm.
pub fn hash_file(path: &Path, algo: HashAlgo) -> Result<Fingerprint> {
    let file = File::open(path).map_err(|e| io_err(path, e))?;
    let mut reader = BufReader::with_capacity(READ_BUF, file);
    let mut buf = vec![0u8; READ_BUF];

    match algo {
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let n = reader.read(&mut buf).map_err(|e| io_err(path, e))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hasher.finalize().to_hex().to_string())
        }
        HashAlgo::Xxh3 => {
            let mut hasher = xxhash_rust::xxh3::Xxh3::new();
            loop {
                let n = reader.read(&mut buf).map_err(|e| io_err(path, e))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(format!("{:032x}", hasher.digest128()))
        }
    }
}

/// Cheap pre-filter: hash only the first `prefix_bytes` of a file. Useful for
/// grouping likely-duplicate candidates before full-content hashing.
pub fn hash_prefix(path: &Path, algo: HashAlgo, prefix_bytes: u64) -> Result<Fingerprint> {
    let file = File::open(path).map_err(|e| io_err(path, e))?;
    let mut reader = BufReader::with_capacity(READ_BUF.min(prefix_bytes as usize + 1).max(1), file)
        .take(prefix_bytes);
    let mut buf = vec![0u8; READ_BUF];

    match algo {
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let n = reader.read(&mut buf).map_err(|e| io_err(path, e))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hasher.finalize().to_hex().to_string())
        }
        HashAlgo::Xxh3 => {
            let mut hasher = xxhash_rust::xxh3::Xxh3::new();
            loop {
                let n = reader.read(&mut buf).map_err(|e| io_err(path, e))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(format!("{:032x}", hasher.digest128()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let p = dir.join(name);
        let mut f = File::create(&p).unwrap();
        f.write_all(bytes).unwrap();
        p
    }

    #[test]
    fn identical_content_blake3_matches() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", b"hello, filecanopy");
        let b = write_file(tmp.path(), "b.bin", b"hello, filecanopy");
        let ha = hash_file(&a, HashAlgo::Blake3).unwrap();
        let hb = hash_file(&b, HashAlgo::Blake3).unwrap();
        assert_eq!(ha, hb);
        assert_eq!(ha.len(), 64); // 256-bit hex
    }

    #[test]
    fn different_content_blake3_differs() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", b"aaaaaaaaaa");
        let b = write_file(tmp.path(), "b.bin", b"bbbbbbbbbb");
        let ha = hash_file(&a, HashAlgo::Blake3).unwrap();
        let hb = hash_file(&b, HashAlgo::Blake3).unwrap();
        assert_ne!(ha, hb);
    }

    #[test]
    fn xxh3_distinguishes_content_and_matches_identical() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", b"payload-1");
        let b = write_file(tmp.path(), "b.bin", b"payload-1");
        let c = write_file(tmp.path(), "c.bin", b"payload-2");
        let ha = hash_file(&a, HashAlgo::Xxh3).unwrap();
        let hb = hash_file(&b, HashAlgo::Xxh3).unwrap();
        let hc = hash_file(&c, HashAlgo::Xxh3).unwrap();
        assert_eq!(ha, hb);
        assert_ne!(ha, hc);
        assert_eq!(ha.len(), 32); // 128-bit hex
    }

    #[test]
    fn empty_file_hashes_to_nonempty_string() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "empty.bin", b"");
        let h = hash_file(&a, HashAlgo::Blake3).unwrap();
        assert!(!h.is_empty(), "blake3 of empty input must be a real digest");
    }

    #[test]
    fn prefix_matches_when_first_bytes_match_but_full_differs() {
        let tmp = tempdir().unwrap();
        let mut common = vec![0u8; 4096];
        for (i, b) in common.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        let mut a_data = common.clone();
        a_data.extend_from_slice(&[1u8; 1024]);
        let mut b_data = common.clone();
        b_data.extend_from_slice(&[2u8; 1024]);

        let a = write_file(tmp.path(), "a.bin", &a_data);
        let b = write_file(tmp.path(), "b.bin", &b_data);

        let pa = hash_prefix(&a, HashAlgo::Blake3, 4096).unwrap();
        let pb = hash_prefix(&b, HashAlgo::Blake3, 4096).unwrap();
        assert_eq!(pa, pb, "prefix hashes should match for identical first 4KiB");

        let fa = hash_file(&a, HashAlgo::Blake3).unwrap();
        let fb = hash_file(&b, HashAlgo::Blake3).unwrap();
        assert_ne!(fa, fb, "full hashes must differ when tails differ");
    }

    #[test]
    fn prefix_short_file_is_consistent() {
        let tmp = tempdir().unwrap();
        let a = write_file(tmp.path(), "a.bin", b"short");
        let b = write_file(tmp.path(), "b.bin", b"short");
        let pa = hash_prefix(&a, HashAlgo::Blake3, 4096).unwrap();
        let pb = hash_prefix(&b, HashAlgo::Blake3, 4096).unwrap();
        assert_eq!(pa, pb);
    }
}
