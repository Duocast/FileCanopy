use iced::widget::{button, column, row, text};
use iced::Element;

use crate::ui::app::App;
use crate::ui::message::Message;
use crate::ui::views::Screen;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = text("Storage overview").size(28);

    let stats: Element<'_, _> = match app.last_scan.as_ref() {
        Some(scan) => row![
            stat_tile("Files", scan.file_count.to_string()),
            stat_tile("Directories", scan.dir_count.to_string()),
            stat_tile("Total size", humansize::format_size(scan.total_bytes, humansize::DECIMAL)),
        ]
        .spacing(16)
        .into(),
        None => text("Run a scan to see your storage overview.").into(),
    };

    let actions = row![
        button(text("Scan a folder…")).on_press(Message::Navigate(Screen::Scan)),
        button(text("Open treemap")).on_press(Message::Navigate(Screen::Treemap)),
        button(text("Find duplicates")).on_press(Message::Navigate(Screen::Duplicates)),
        button(text("Export report")).on_press(Message::Navigate(Screen::Reports)),
    ]
    .spacing(8);

    column![header, stats, actions].spacing(16).into()
}

fn stat_tile(label: &'static str, value: String) -> Element<'static, Message> {
    column![text(label).size(14), text(value).size(22)].spacing(4).into()
}
