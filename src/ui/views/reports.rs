use iced::widget::{button, column, pick_list, row, text};
use iced::Element;

use crate::reports::ExportFormat;
use crate::ui::app::App;
use crate::ui::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let path_label = app
        .export_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(no destination chosen)".into());

    let format_picker = pick_list(
        ExportFormat::ALL
            .iter()
            .copied()
            .map(FormatChoice)
            .collect::<Vec<_>>(),
        Some(FormatChoice(app.export_format)),
        |c| Message::ExportFormatChanged(c.0),
    );

    let start = {
        let mut b = button(text("Export"));
        if app.last_scan.is_some() && app.export_path.is_some() {
            b = b.on_press(Message::StartExport);
        }
        b
    };

    let mut col = column![
        text("Export report").size(24),
        row![text("Format:"), format_picker].spacing(8),
        row![
            text("Destination:"),
            text(path_label),
            button(text("Pick…")).on_press(Message::PickExportPath),
        ]
        .spacing(8),
        start,
    ]
    .spacing(12);

    if let Some(path) = &app.last_export {
        col = col.push(text(format!("Wrote {}", path.display())));
    }

    col.into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FormatChoice(ExportFormat);

impl std::fmt::Display for FormatChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.label())
    }
}
