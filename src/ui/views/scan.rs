use iced::widget::{button, column, progress_bar, row, text};
use iced::Element;

use crate::ui::app::App;
use crate::ui::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let root_label = app
        .scan_root
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(no folder selected)".into());

    let pick = button(text("Choose folder…")).on_press(Message::PickScanRoot);

    let start = {
        let mut b = button(text("Start scan"));
        if app.scan_root.is_some() && !app.scan_in_progress {
            b = b.on_press(Message::StartScan);
        }
        b
    };

    let cancel = {
        let mut b = button(text("Cancel"));
        if app.scan_in_progress {
            b = b.on_press(Message::CancelScan);
        }
        b
    };

    let mut col = column![
        text("Scan a directory").size(24),
        row![text("Root:"), text(root_label)].spacing(8),
        row![pick, start, cancel].spacing(8),
    ]
    .spacing(12);

    if app.scan_in_progress {
        // Indeterminate-ish progress bar driven off file count.
        col = col.push(progress_bar(0.0..=1.0, 0.5));
        col = col.push(text(format!(
            "Scanned {} files, {}",
            app.scan_progress.files_seen,
            humansize::format_size(app.scan_progress.bytes_seen, humansize::DECIMAL),
        )));
        if let Some(p) = &app.scan_progress.current_path {
            col = col.push(text(p.display().to_string()).size(12));
        }
    }

    if let Some(err) = &app.last_error {
        col = col.push(text(format!("Error: {err}")));
    }

    if let Some(report) = &app.last_scan {
        col = col.push(text(format!(
            "Last scan: {} files, {}",
            report.file_count,
            humansize::format_size(report.total_bytes, humansize::DECIMAL)
        )));
    }

    col.into()
}
