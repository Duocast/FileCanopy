use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::ui::message::Message;
use crate::ui::views::Screen;

pub fn view(active: Screen) -> Element<'static, Message> {
    let mut col = column![text("FileCanopy").size(22)].spacing(8).padding(12);

    for screen in Screen::ALL {
        let mut btn = button(text(screen.label()))
            .width(Length::Fill)
            .on_press(Message::Navigate(*screen));
        if *screen == active {
            btn = btn.style(button::primary);
        } else {
            btn = btn.style(button::secondary);
        }
        col = col.push(btn);
    }

    container(col).width(Length::Fixed(200.0)).height(Length::Fill).into()
}
