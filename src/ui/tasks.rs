//! Async wrappers that bridge library operations into `iced::Task`s.
//!
//! Everything that touches the filesystem runs off the UI thread. Tasks return
//! a `Message` so the `App::update` reducer can apply the result.

use std::path::PathBuf;
use std::sync::Arc;

use iced::Task;

use crate::analysis::DuplicatesReport;
use crate::analysis::hasher::HashAlgo;
use crate::dedup::DedupStrategy;
use crate::reports::ExportFormat;
use crate::scanner::{ScanOptions, ScanReport};
use crate::ui::message::Message;

/// Kick off a scan in the background. Progress is reported via the
/// subscription wired up in [`App::subscription`].
pub fn scan(opts: ScanOptions) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || crate::scanner::walker::scan(&opts))
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r.map_err(|e| e.to_string()))
        },
        |res| match res {
            Ok(report) => Message::ScanFinished(Arc::new(report)),
            Err(err) => Message::ScanFailed(err),
        },
    )
}

pub fn find_duplicates(report: Arc<ScanReport>, min_size: u64, algo: HashAlgo) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                crate::analysis::duplicates::find(&report, min_size, algo)
            })
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r.map_err(|e| e.to_string()))
        },
        |res| match res {
            Ok(d) => Message::DuplicatesFound(Arc::new(d)),
            Err(err) => Message::DuplicatesFailed(err),
        },
    )
}

pub fn apply_dedup(report: Arc<DuplicatesReport>, strategy: DedupStrategy) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || crate::dedup::apply(&report, strategy))
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r.map_err(|e| e.to_string()))
        },
        |res| match res {
            Ok(outcome) => Message::DedupFinished(outcome),
            // We surface the failure through the duplicates channel for now;
            // a dedicated DedupFailed message can be added if needed.
            Err(err) => Message::DuplicatesFailed(err),
        },
    )
}

pub fn export(report: Arc<ScanReport>, path: PathBuf, format: ExportFormat) -> Task<Message> {
    Task::perform(
        async move {
            let path_for_task = path.clone();
            tokio::task::spawn_blocking(move || {
                crate::reports::export(&report, &path_for_task, Some(format))
                    .map(|_| path_for_task)
            })
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r.map_err(|e| e.to_string()))
        },
        Message::ExportFinished,
    )
}

/// Open the native folder picker and return the chosen path.
pub fn pick_folder() -> Task<Message> {
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .set_title("Select a folder to scan")
                .pick_folder()
                .await
                .map(|h| h.path().to_path_buf())
        },
        Message::ScanRootPicked,
    )
}

/// Open the native save-file picker for exports.
pub fn pick_save(default_ext: &'static str) -> Task<Message> {
    Task::perform(
        async move {
            rfd::AsyncFileDialog::new()
                .set_title("Save report as")
                .add_filter(default_ext, &[default_ext])
                .save_file()
                .await
                .map(|h| h.path().to_path_buf())
        },
        Message::ExportPathPicked,
    )
}
