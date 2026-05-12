use chrono::{DateTime, Local};
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::scanner::FileEntry;
use crate::ui::app::App;
use crate::ui::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = text(format!("Top {} largest files", app.largest_limit)).size(24);

    let body: Element<'_, _> = match app.last_scan.as_ref() {
        Some(report) => {
            let entries =
                crate::analysis::largest::top(report, app.largest_limit, app.largest_min_size);

            let selected_path = app.largest_selected.as_deref();
            let mut list = column![].spacing(2);
            for entry in &entries {
                let is_selected = selected_path == Some(entry.path.as_path());
                let label = row![
                    text(humansize::format_size(entry.size, humansize::DECIMAL))
                        .width(Length::Fixed(120.0)),
                    text(entry.path.display().to_string()),
                ];
                let mut btn = button(label)
                    .padding([4, 8])
                    .width(Length::Fill)
                    .on_press(Message::LargestFileSelected(entry.path.clone()));
                btn = if is_selected {
                    btn.style(button::primary)
                } else {
                    btn.style(button::text)
                };
                list = list.push(btn);
            }

            let list = scrollable(list).height(Length::Fill).width(Length::FillPortion(3));

            let selected_entry = selected_path
                .and_then(|p| entries.iter().find(|e| e.path.as_path() == p))
                .cloned();
            let details = details_panel(selected_entry.as_ref());

            row![list, details].spacing(16).into()
        }
        None => text("Run a scan to see the largest files.").into(),
    };

    column![header, body].spacing(12).into()
}

fn details_panel(selected: Option<&FileEntry>) -> Element<'static, Message> {
    let inner: Element<'_, _> = match selected {
        Some(entry) => {
            let name = entry
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| entry.path.display().to_string());
            let parent = entry
                .path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            let extension = entry
                .path
                .extension()
                .map(|e| e.to_string_lossy().into_owned())
                .unwrap_or_else(|| "—".into());
            let size_human = humansize::format_size(entry.size, humansize::DECIMAL);
            let size_bytes = format!("{} bytes", entry.size);
            let size_on_disk = entry
                .size_on_disk
                .map(|s| {
                    format!(
                        "{} ({} bytes)",
                        humansize::format_size(s, humansize::DECIMAL),
                        s
                    )
                })
                .unwrap_or_else(|| "—".into());
            let modified = entry
                .modified
                .map(|t| {
                    let dt: DateTime<Local> = t.into();
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|| "—".into());
            let lines = entry
                .line_count
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into());

            column![
                text("File details").size(18),
                Space::new().height(Length::Fixed(4.0)),
                detail_field("Name", name),
                detail_field("Parent", parent),
                detail_field("Extension", extension),
                detail_field("Size", size_human),
                detail_field("Size (exact)", size_bytes),
                detail_field("Size on disk", size_on_disk),
                detail_field("Modified", modified),
                detail_field("Line count", lines),
                detail_field("Full path", entry.path.display().to_string()),
            ]
            .spacing(6)
            .into()
        }
        None => column![
            text("File details").size(18),
            Space::new().height(Length::Fixed(4.0)),
            text("Click a file in the list to see details.").size(13),
        ]
        .spacing(6)
        .into(),
    };

    container(inner)
        .padding(12)
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .into()
}

fn detail_field(label: &'static str, value: String) -> Element<'static, Message> {
    column![
        text(label).size(11),
        text(value).size(13),
    ]
    .spacing(2)
    .into()
}
