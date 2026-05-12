//! FileCanopy — disk space analyzer, duplicate finder, and report generator.
//!
//! Top-level module map:
//! - [`cli`]        — argument parsing and command dispatch
//! - [`scanner`]    — directory traversal and metadata collection
//! - [`analysis`]   — tree construction, ranking, duplicate detection
//! - [`visualization`] — treemap and chart rendering
//! - [`reports`]    — PDF / Excel / HTML / CSV export
//! - [`history`]    — snapshot storage and over-time comparison
//! - [`scheduler`]  — OS-native scheduled task installation
//! - [`dedup`]      — deduplication actions (delete, hardlink, symlink)
//! - [`platform`]   — Windows/Linux abstractions
//! - [`config`]     — user configuration
//! - [`error`]      — crate-wide error type
//! - [`telemetry`]  — tracing initialization

pub mod analysis;
pub mod cli;
pub mod config;
pub mod dedup;
pub mod error;
pub mod history;
pub mod platform;
pub mod reports;
pub mod scanner;
pub mod scheduler;
pub mod telemetry;
pub mod visualization;

pub use error::{Error, Result};
