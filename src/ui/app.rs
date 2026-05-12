//! Top-level application state, update reducer, and view dispatcher.

use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{container, row};
use iced::{Element, Length, Subscription, Task, Theme};

use crate::analysis::{DuplicatesReport, SizeTree};
use crate::config::Config;
use crate::dedup::{DedupOutcome, DedupStrategy};
use crate::history::{DiffReport, Snapshot};
use crate::reports::ExportFormat;
use crate::scanner::ScanReport;
use crate::ui::message::{Message, ScanProgress};
use crate::ui::theme::ThemeChoice;
use crate::ui::views::{self, Screen};

#[derive(Default)]
pub struct App {
    pub screen: Screen,
    pub config: Config,
    pub theme: ThemeChoice,

    // --- Scan ---
    pub scan_root: Option<PathBuf>,
    pub scan_in_progress: bool,
    pub scan_progress: ScanProgress,
    pub last_scan: Option<Arc<ScanReport>>,
    pub last_size_tree: Option<Arc<SizeTree>>,
    pub last_error: Option<String>,

    // --- Duplicates / dedupe ---
    pub duplicates: Option<Arc<DuplicatesReport>>,
    pub dedup_strategy: DedupStrategy,
    pub dedup_outcome: Option<DedupOutcome>,

    // --- Largest files ---
    pub largest_limit: usize,
    pub largest_min_size: Option<u64>,

    // --- Line counter ---
    pub line_count_extensions: Vec<(String, bool)>,
    pub line_count_threshold: usize,
    pub last_line_count: Option<Arc<crate::scanner::line_counter::LineCountReport>>,

    // --- History ---
    pub snapshot_label: String,
    pub snapshots: Vec<Snapshot>,
    pub compare_before: Option<String>,
    pub compare_after: Option<String>,
    pub last_diff: Option<Arc<DiffReport>>,

    // --- Reports / export ---
    pub export_path: Option<PathBuf>,
    pub export_format: ExportFormat,
    pub last_export: Option<PathBuf>,

    // --- Scheduler ---
    pub schedule_cron: String,
    pub schedule_name: String,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let config = Config::load().unwrap_or_default();
        let app = App {
            largest_limit: 50,
            line_count_extensions: default_extensions(),
            line_count_threshold: 1000,
            export_format: ExportFormat::Pdf,
            schedule_name: "filecanopy".into(),
            config,
            ..Self::default()
        };
        (app, Task::none())
    }

    pub fn title(&self) -> String {
        match self.screen {
            Screen::Dashboard => "FileCanopy".into(),
            other => format!("FileCanopy — {}", other.label()),
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme.to_iced()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // TODO: hook up a channel-based subscription so scanner workers can
        // emit `Message::ScanProgress` as they walk the tree.
        Subscription::none()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(screen) => {
                self.screen = screen;
                Task::none()
            }

            // --- Scan ---
            Message::PickScanRoot => crate::ui::tasks::pick_folder(),
            Message::ScanRootPicked(path) => {
                self.scan_root = path;
                Task::none()
            }
            Message::StartScan => {
                let Some(root) = self.scan_root.clone() else {
                    return Task::none();
                };
                self.scan_in_progress = true;
                self.scan_progress = ScanProgress::default();
                self.last_error = None;
                crate::ui::tasks::scan(crate::scanner::ScanOptions {
                    roots: vec![root],
                    follow_symlinks: self.config.follow_symlinks,
                    excludes: self.config.ignore_globs.clone(),
                    ..Default::default()
                })
            }
            Message::ScanProgress(p) => {
                self.scan_progress = p;
                Task::none()
            }
            Message::ScanFinished(report) => {
                self.scan_in_progress = false;
                let roots: Vec<PathBuf> = self.scan_root.iter().cloned().collect();
                self.last_size_tree = crate::analysis::tree::build(&report, &roots)
                    .ok()
                    .map(Arc::new);
                self.last_scan = Some(report);
                Task::none()
            }
            Message::ScanFailed(err) => {
                self.scan_in_progress = false;
                self.last_error = Some(err);
                Task::none()
            }
            Message::CancelScan => {
                // TODO: wire a cancel token through `tasks::scan`
                self.scan_in_progress = false;
                Task::none()
            }

            // --- Treemap (placeholders) ---
            Message::TreemapTileClicked(_) => Task::none(),
            Message::TreemapZoomIn => Task::none(),
            Message::TreemapZoomOut => Task::none(),

            // --- Largest ---
            Message::LargestLimitChanged(n) => {
                self.largest_limit = n;
                Task::none()
            }
            Message::LargestMinSizeChanged(min) => {
                self.largest_min_size = min;
                Task::none()
            }

            // --- Duplicates / dedup ---
            Message::FindDuplicates => {
                let Some(report) = self.last_scan.clone() else {
                    return Task::none();
                };
                crate::ui::tasks::find_duplicates(report, 1024, Default::default())
            }
            Message::DuplicatesFound(d) => {
                self.duplicates = Some(d);
                Task::none()
            }
            Message::DuplicatesFailed(err) => {
                self.last_error = Some(err);
                Task::none()
            }
            Message::DedupStrategyChanged(s) => {
                self.dedup_strategy = s;
                Task::none()
            }
            Message::ApplyDedup => {
                let Some(d) = self.duplicates.clone() else {
                    return Task::none();
                };
                crate::ui::tasks::apply_dedup(d, self.dedup_strategy)
            }
            Message::DedupFinished(outcome) => {
                self.dedup_outcome = Some(outcome);
                Task::none()
            }

            // --- Line counter ---
            Message::LineCountExtToggled(ext, on) => {
                if let Some(entry) = self.line_count_extensions.iter_mut().find(|(e, _)| e == &ext) {
                    entry.1 = on;
                } else if on {
                    self.line_count_extensions.push((ext, true));
                }
                Task::none()
            }
            Message::LineCountThresholdChanged(t) => {
                self.line_count_threshold = t;
                Task::none()
            }
            Message::RunLineCount => {
                // TODO: wire to scanner::line_counter::count via a Task
                Task::none()
            }
            Message::LineCountFinished(r) => {
                self.last_line_count = Some(r);
                Task::none()
            }

            // --- History ---
            Message::SnapshotLabelChanged(s) => {
                self.snapshot_label = s;
                Task::none()
            }
            Message::TakeSnapshot => Task::none(), // TODO: history::snapshot::take
            Message::SnapshotTaken(snap) => {
                self.snapshots.push(snap);
                Task::none()
            }
            Message::SnapshotSelectedBefore(s) => {
                self.compare_before = Some(s);
                Task::none()
            }
            Message::SnapshotSelectedAfter(s) => {
                self.compare_after = Some(s);
                Task::none()
            }
            Message::CompareSnapshots => Task::none(), // TODO: history::compare::diff
            Message::CompareFinished(d) => {
                self.last_diff = Some(d);
                Task::none()
            }

            // --- Reports ---
            Message::PickExportPath => {
                crate::ui::tasks::pick_save(self.export_format.extension())
            }
            Message::ExportPathPicked(p) => {
                self.export_path = p;
                Task::none()
            }
            Message::ExportFormatChanged(f) => {
                self.export_format = f;
                Task::none()
            }
            Message::StartExport => {
                let (Some(report), Some(path)) =
                    (self.last_scan.clone(), self.export_path.clone())
                else {
                    return Task::none();
                };
                crate::ui::tasks::export(report, path, self.export_format)
            }
            Message::ExportFinished(Ok(path)) => {
                self.last_export = Some(path);
                Task::none()
            }
            Message::ExportFinished(Err(err)) => {
                self.last_error = Some(err);
                Task::none()
            }

            // --- Scheduler ---
            Message::ScheduleCronChanged(s) => {
                self.schedule_cron = s;
                Task::none()
            }
            Message::ScheduleNameChanged(s) => {
                self.schedule_name = s;
                Task::none()
            }
            Message::InstallSchedule => Task::none(), // TODO: scheduler::install
            Message::RemoveSchedule(_name) => Task::none(), // TODO: scheduler::remove
            Message::ScheduleUpdated => Task::none(),

            // --- Settings ---
            Message::ToggleFollowSymlinks(v) => {
                self.config.follow_symlinks = v;
                Task::none()
            }
            Message::AddIgnoreGlob(g) => {
                self.config.ignore_globs.push(g);
                Task::none()
            }
            Message::RemoveIgnoreGlob(i) => {
                if i < self.config.ignore_globs.len() {
                    self.config.ignore_globs.remove(i);
                }
                Task::none()
            }
            Message::SaveSettings => {
                let _ = self.config.save();
                Task::none()
            }

            Message::NoOp => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = match self.screen {
            Screen::Dashboard => views::dashboard::view(self),
            Screen::Scan => views::scan::view(self),
            Screen::Treemap => views::treemap::view(self),
            Screen::Largest => views::largest::view(self),
            Screen::Duplicates => views::duplicates::view(self),
            Screen::LineCount => views::line_count::view(self),
            Screen::History => views::history::view(self),
            Screen::Reports => views::reports::view(self),
            Screen::Scheduler => views::scheduler::view(self),
            Screen::Settings => views::settings::view(self),
        };

        let sidebar = views::sidebar::view(self.screen);

        container(row![sidebar, container(body).padding(16).width(Length::Fill)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn default_extensions() -> Vec<(String, bool)> {
    ["rs", "py", "ts", "tsx", "js", "jsx", "go", "java", "c", "cpp", "h", "hpp"]
        .iter()
        .map(|e| ((*e).to_string(), false))
        .collect()
}

