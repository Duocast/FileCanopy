use iced::widget::{button, checkbox, column, row, text};
use iced::Element;

use crate::ui::app::App;
use crate::ui::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let follow = checkbox(app.config.follow_symlinks)
        .label("Follow symbolic links")
        .on_toggle(Message::ToggleFollowSymlinks);

    let mut ignores = column![text("Ignore globs").size(16)].spacing(4);
    for (i, glob) in app.config.ignore_globs.iter().enumerate() {
        ignores = ignores.push(row![
            text(glob.clone()),
            button(text("remove")).on_press(Message::RemoveIgnoreGlob(i)),
        ].spacing(8));
    }

    column![
        text("Settings").size(24),
        follow,
        ignores,
        button(text("Save")).on_press(Message::SaveSettings),
    ]
    .spacing(12)
    .into()
}
