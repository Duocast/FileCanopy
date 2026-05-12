use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Result;

/// Persistent user configuration. Loaded from `~/.config/filecanopy/config.toml`
/// on Linux, `%APPDATA%\filecanopy\config.toml` on Windows.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub default_roots: Vec<PathBuf>,
    pub snapshot_db: Option<PathBuf>,
    pub report_dir: Option<PathBuf>,
    pub ignore_globs: Vec<String>,
    pub follow_symlinks: bool,
    pub line_counting: LineCountingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LineCountingConfig {
    /// File extensions to count lines for. Empty = no line counting.
    pub extensions: Vec<String>,
    /// Skip files larger than this many bytes when counting lines.
    pub max_size_bytes: Option<u64>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // TODO: read TOML from platform config dir
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<()> {
        // TODO: write TOML to platform config dir
        Ok(())
    }

    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("filecanopy").join("config.toml"))
    }
}
