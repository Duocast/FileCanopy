use iced::widget::{button, column, row, text, text_input};
use iced::Element;

use crate::ui::app::App;
use crate::ui::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let form = column![
        row![
            text("Name"),
            text_input("filecanopy", &app.schedule_name).on_input(Message::ScheduleNameChanged),
        ]
        .spacing(8),
        row![
            text("Cron / interval"),
            text_input("0 2 * * *", &app.schedule_cron).on_input(Message::ScheduleCronChanged),
        ]
        .spacing(8),
        button(text("Install scheduled task")).on_press(Message::InstallSchedule),
    ]
    .spacing(8);

    column![
        text("Scheduler").size(24),
        text(
            "Recurring scans run in the background via the host OS scheduler \
             (cron on Linux, Task Scheduler on Windows)."
        ),
        form,
    ]
    .spacing(12)
    .into()
}
