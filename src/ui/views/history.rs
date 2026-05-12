use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::ui::app::App;
use crate::ui::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let snapshot_row = row![
        text_input("Snapshot label", &app.snapshot_label).on_input(Message::SnapshotLabelChanged),
        button(text("Take snapshot")).on_press(Message::TakeSnapshot),
    ]
    .spacing(8);

    let mut list = column![text(format!("Snapshots ({})", app.snapshots.len())).size(16)].spacing(4);
    for snap in &app.snapshots {
        list = list.push(text(format!(
            "{} — {} ({})",
            snap.id,
            snap.label.clone().unwrap_or_default(),
            snap.taken_at,
        )));
    }

    let compare_row = row![
        button(text("Compare selected")).on_press(Message::CompareSnapshots),
    ];

    let diff: Element<'_, _> = match app.last_diff.as_ref() {
        Some(d) => column![
            text(format!("Δ total: {} bytes", d.total_delta_bytes)),
            text(format!(
                "added {}  removed {}  grown {}  shrunk {}",
                d.added.len(),
                d.removed.len(),
                d.grown.len(),
                d.shrunk.len(),
            )),
        ]
        .spacing(4)
        .into(),
        None => text("Select two snapshots and compare.").into(),
    };

    column![
        text("History").size(24),
        snapshot_row,
        scrollable(list).height(Length::Fixed(200.0)),
        compare_row,
        diff,
    ]
    .spacing(12)
    .into()
}
