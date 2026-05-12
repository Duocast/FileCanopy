use iced::widget::{column, container, image, text};
use iced::{Alignment, ContentFit, Element, Length};

use crate::ui::app::App;
use crate::ui::message::Message;

const LOGO_BYTES: &[u8] =
    include_bytes!("../../../Assets/FileCanopyLogov1-transparent.png");

pub fn view(_app: &App) -> Element<'_, Message> {
    let handle = image::Handle::from_bytes(LOGO_BYTES.to_vec());

    let logo = image(handle)
        .content_fit(ContentFit::Contain)
        .width(Length::Fixed(320.0))
        .height(Length::Fixed(240.0));

    let body = column![
        logo,
        text("FileCanopy").size(32),
        text(format!("Version {}", env!("CARGO_PKG_VERSION"))).size(16),
        text(env!("CARGO_PKG_DESCRIPTION")).size(14),
        text("Licensed under MIT OR Apache-2.0").size(12),
        text(concat!("Repository: ", env!("CARGO_PKG_REPOSITORY"))).size(12),
    ]
    .spacing(12)
    .align_x(Alignment::Center);

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
