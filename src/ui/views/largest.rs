use iced::widget::{column, row, scrollable, text};
use iced::{Element, Length};

use crate::ui::app::App;
use crate::ui::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = text(format!("Top {} largest files", app.largest_limit)).size(24);

    let body: Element<'_, _> = match app.last_scan.as_ref() {
        Some(report) => {
            let entries =
                crate::analysis::largest::top(report, app.largest_limit, app.largest_min_size);
            let mut col = column![].spacing(2);
            for entry in entries {
                col = col.push(row![
                    text(humansize::format_size(entry.size, humansize::DECIMAL))
                        .width(Length::Fixed(120.0)),
                    text(entry.path.display().to_string()),
                ]);
            }
            scrollable(col).height(Length::Fill).into()
        }
        None => text("Run a scan to see the largest files.").into(),
    };

    column![header, body].spacing(12).into()
}
