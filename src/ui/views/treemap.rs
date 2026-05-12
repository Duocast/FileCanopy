use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use crate::ui::app::App;
use crate::ui::message::Message;
use crate::ui::widgets::treemap_canvas::TreemapCanvas;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = row![
        text("Treemap").size(24),
        button(text("−")).on_press(Message::TreemapZoomOut),
        button(text("+")).on_press(Message::TreemapZoomIn),
    ]
    .spacing(8);

    let body: Element<'_, _> = match app.last_scan.as_ref() {
        Some(_report) => container(TreemapCanvas::new(app).into_element())
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        None => text("Run a scan first — the treemap reflects the most recent scan.").into(),
    };

    column![header, body].spacing(12).into()
}
