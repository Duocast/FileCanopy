use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use crate::ui::app::{App, find_dir};
use crate::ui::message::Message;
use crate::ui::widgets::treemap_canvas::TreemapCanvas;

pub fn view(app: &App) -> Element<'_, Message> {
    let title = text("Treemap").size(24);
    let zoom_out = button(text("−"))
        .on_press(Message::TreemapZoomOut)
        .padding([4, 10]);
    let zoom_in = button(text("+"))
        .on_press(Message::TreemapZoomIn)
        .padding([4, 10]);

    let Some(tree) = app.last_size_tree.as_ref() else {
        return column![
            row![title].spacing(8),
            text("Run a scan first — the treemap reflects the most recent scan."),
        ]
        .spacing(12)
        .into();
    };

    let focus_node = app
        .treemap_focus
        .as_deref()
        .and_then(|p| find_dir(&tree.root, p))
        .unwrap_or(&tree.root);

    let focus_label = focus_node.path.display().to_string();
    let total = humansize::format_size(focus_node.size, humansize::DECIMAL);
    let info = text(format!(
        "{}  ·  {}  ·  {} files  ·  showing up to {} tiles",
        focus_label, total, focus_node.file_count, app.treemap_max_tiles
    ))
    .size(13);

    let mut nav = row![title, zoom_out, zoom_in].spacing(8).align_y(iced::Alignment::Center);
    if app.treemap_focus.is_some() {
        nav = nav.push(
            button(text("↑ Up"))
                .on_press(Message::TreemapFocusUp)
                .padding([4, 10]),
        );
        nav = nav.push(
            button(text("⌂ Root"))
                .on_press(Message::TreemapFocusRoot)
                .padding([4, 10]),
        );
    }

    let body: Element<'_, _> = container(TreemapCanvas::new(app).into_element())
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    column![nav, info, body].spacing(8).into()
}
