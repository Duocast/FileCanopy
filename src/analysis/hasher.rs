use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Result;

/// Stable 256-bit content fingerprint, base16-encoded.
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

/// Hash a file's full contents with the chosen algorithm.
pub fn hash_file(_path: &Path, _algo: HashAlgo) -> Result<Fingerprint> {
    // TODO: streaming blake3 / xxh3 over a mmap or buffered reader
    Ok(String::new())
}

/// Cheap pre-filter: hash only the first `prefix_bytes` of a file. Useful for
/// grouping likely-duplicate candidates before full-content hashing.
pub fn hash_prefix(_path: &Path, _algo: HashAlgo, _prefix_bytes: u64) -> Result<Fingerprint> {
    // TODO
    Ok(String::new())
}
