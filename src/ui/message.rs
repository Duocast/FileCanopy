//! Every event the UI can emit. Keep this enum flat — nesting per-screen
//! sub-enums quickly becomes painful with iced's `map` ergonomics.

use std::path::PathBuf;
use std::sync::Arc;

use crate::analysis::DuplicatesReport;
use crate::dedup::{DedupOutcome, DedupStrategy};
use crate::history::{DiffReport, Snapshot};
use crate::reports::ExportFormat;
use crate::scanner::ScanReport;
use crate::ui::views::Screen;

#[derive(Debug, Clone)]
pub enum Message {
    /// Sidebar navigation.
    Navigate(Screen),

    // --- Scan ---
    PickScanRoot,
    ScanRootPicked(Option<PathBuf>),
    StartScan,
    ScanProgress(ScanProgress),
    ScanFinished(Arc<ScanReport>),
    ScanFailed(String),
    CancelScan,

    // --- Treemap ---
    /// Drill into a subdirectory by clicking its tile, or drill up one level
    /// when invoked with the parent of the current focus.
    TreemapTileClicked(PathBuf),
    /// Pop the focus up one level (toward the scan root). Bound to the "Up"
    /// button shown in the treemap breadcrumb.
    TreemapFocusUp,
    /// Reset focus back to the scan root.
    TreemapFocusRoot,
    /// Show more detail (more tiles, smaller minimum-size threshold).
    TreemapZoomIn,
    /// Show less detail (fewer tiles, larger long-tail aggregate).
    TreemapZoomOut,

    // --- Largest files / "space hogs" ---
    LargestLimitChanged(usize),
    LargestMinSizeChanged(Option<u64>),

    // --- Duplicates / dedupe ---
    FindDuplicates,
    DuplicatesFound(Arc<DuplicatesReport>),
    DuplicatesFailed(String),
    DedupStrategyChanged(DedupStrategy),
    ApplyDedup,
    DedupFinished(DedupOutcome),

    // --- Line counter (code-repo assessments) ---
    LineCountExtToggled(String, bool),
    LineCountThresholdChanged(usize),
    RunLineCount,
    LineCountFinished(Arc<crate::scanner::line_counter::LineCountReport>),

    // --- History / over-time comparison ---
    SnapshotLabelChanged(String),
    TakeSnapshot,
    SnapshotTaken(Snapshot),
    SnapshotSelectedBefore(String),
    SnapshotSelectedAfter(String),
    CompareSnapshots,
    CompareFinished(Arc<DiffReport>),

    // --- Reports / export ---
    PickExportPath,
    ExportPathPicked(Option<PathBuf>),
    ExportFormatChanged(ExportFormat),
    StartExport,
    ExportFinished(Result<PathBuf, String>),

    // --- Scheduler ---
    ScheduleCronChanged(String),
    ScheduleNameChanged(String),
    InstallSchedule,
    RemoveSchedule(String),
    ScheduleUpdated,

    // --- Settings ---
    ToggleFollowSymlinks(bool),
    AddIgnoreGlob(String),
    RemoveIgnoreGlob(usize),
    SaveSettings,

    /// No-op; useful as a placeholder return from async tasks.
    NoOp,
}

/// Progress emitted while a scan is running. Sent often, so kept cheap to clone.
#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub files_seen: u64,
    pub bytes_seen: u64,
    pub current_path: Option<PathBuf>,
}
