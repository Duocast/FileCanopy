use iced::widget::{button, checkbox, column, row, scrollable, text};
use iced::{Element, Length};

use crate::ui::app::App;
use crate::ui::message::Message;

pub fn view(app: &App) -> Element<'_, Message> {
    let mut ext_row = row![].spacing(8);
    for (ext, enabled) in &app.line_count_extensions {
        let ext_clone = ext.clone();
        ext_row = ext_row.push(checkbox(*enabled).label(ext.clone()).on_toggle(
            move |on| Message::LineCountExtToggled(ext_clone.clone(), on),
        ));
    }

    let run = {
        let mut b = button(text("Count lines"));
        if app.last_scan.is_some() {
            b = b.on_press(Message::RunLineCount);
        }
        b
    };

    let body: Element<'_, _> = match app.last_line_count.as_ref() {
        Some(report) => {
            let mut col = column![text(format!("Total lines: {}", report.total_lines))].spacing(4);
            if !report.monolithic.is_empty() {
                col = col.push(
                    text(format!(
                        "Monolithic files (≥ {} lines):",
                        app.line_count_threshold
                    ))
                    .size(16),
                );
                for f in &report.monolithic {
                    col = col.push(text(format!("{} — {}", f.lines, f.path.display())));
                }
            }
            scrollable(col).height(Length::Fill).into()
        }
        None => text("Toggle extensions and run a count over the most recent scan.").into(),
    };

    column![text("Line count").size(24), ext_row, run, body].spacing(12).into()
}
