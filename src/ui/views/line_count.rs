use iced::widget::{button, checkbox, column, row, scrollable, text};
use iced::{Element, Length};

use crate::scanner::line_counter::FileLineCount;
use crate::ui::app::App;
use crate::ui::message::Message;

/// Column the file table is currently sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCountSortColumn {
    Path,
    Lines,
    CodeLines,
    CommentLines,
    Bytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

impl Default for LineCountSortColumn {
    fn default() -> Self {
        LineCountSortColumn::Lines
    }
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Descending
    }
}

const COL_LINES: f32 = 110.0;
const COL_CODE: f32 = 110.0;
const COL_COMMENT: f32 = 130.0;
const COL_BYTES: f32 = 110.0;

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
            let comment_lines = report.total_lines.saturating_sub(report.total_code_lines);
            let summary = column![
                text(format!(
                    "Total lines: {}  •  Total code lines: {}  •  Comment/blank: {}  •  Files: {}",
                    report.total_lines,
                    report.total_code_lines,
                    comment_lines,
                    report.per_file.len(),
                )),
            ]
            .spacing(4);

            let sort_col = app.line_count_sort_column;
            let sort_dir = app.line_count_sort_direction;
            let header = table_header(sort_col, sort_dir);

            let mut rows = sorted_rows(&report.per_file, sort_col, sort_dir);
            let threshold = app.line_count_threshold as u64;

            let mut list = column![].spacing(2);
            for f in rows.drain(..) {
                list = list.push(file_row(&f, threshold));
            }

            column![
                summary,
                header,
                scrollable(list).height(Length::Fill),
            ]
            .spacing(8)
            .into()
        }
        None => text("Toggle extensions and run a count over the most recent scan.").into(),
    };

    column![text("Line count").size(24), ext_row, run, body].spacing(12).into()
}

fn sorted_rows(
    files: &[FileLineCount],
    col: LineCountSortColumn,
    dir: SortDirection,
) -> Vec<FileLineCount> {
    let mut out: Vec<FileLineCount> = files.to_vec();
    out.sort_by(|a, b| {
        let ord = match col {
            LineCountSortColumn::Path => a.path.cmp(&b.path),
            LineCountSortColumn::Lines => a.lines.cmp(&b.lines),
            LineCountSortColumn::CodeLines => a.code_lines.cmp(&b.code_lines),
            LineCountSortColumn::CommentLines => {
                let ac = a.lines.saturating_sub(a.code_lines);
                let bc = b.lines.saturating_sub(b.code_lines);
                ac.cmp(&bc)
            }
            LineCountSortColumn::Bytes => a.bytes.cmp(&b.bytes),
        };
        match dir {
            SortDirection::Ascending => ord,
            SortDirection::Descending => ord.reverse(),
        }
    });
    out
}

fn table_header(
    active: LineCountSortColumn,
    dir: SortDirection,
) -> Element<'static, Message> {
    row![
        header_button("Total", LineCountSortColumn::Lines, active, dir, COL_LINES),
        header_button("Code", LineCountSortColumn::CodeLines, active, dir, COL_CODE),
        header_button(
            "Comment/blank",
            LineCountSortColumn::CommentLines,
            active,
            dir,
            COL_COMMENT,
        ),
        header_button("Bytes", LineCountSortColumn::Bytes, active, dir, COL_BYTES),
        header_button_fill("Path", LineCountSortColumn::Path, active, dir),
    ]
    .spacing(4)
    .into()
}

fn header_button(
    label: &str,
    col: LineCountSortColumn,
    active: LineCountSortColumn,
    dir: SortDirection,
    width: f32,
) -> Element<'static, Message> {
    button(text(header_label(label, col, active, dir)).size(13))
        .padding([4, 8])
        .width(Length::Fixed(width))
        .on_press(Message::LineCountSortBy(col))
        .style(button::secondary)
        .into()
}

fn header_button_fill(
    label: &str,
    col: LineCountSortColumn,
    active: LineCountSortColumn,
    dir: SortDirection,
) -> Element<'static, Message> {
    button(text(header_label(label, col, active, dir)).size(13))
        .padding([4, 8])
        .width(Length::Fill)
        .on_press(Message::LineCountSortBy(col))
        .style(button::secondary)
        .into()
}

fn header_label(
    label: &str,
    col: LineCountSortColumn,
    active: LineCountSortColumn,
    dir: SortDirection,
) -> String {
    if col == active {
        let arrow = match dir {
            SortDirection::Ascending => "↑",
            SortDirection::Descending => "↓",
        };
        format!("{label} {arrow}")
    } else {
        label.to_string()
    }
}

fn file_row(f: &FileLineCount, threshold: u64) -> Element<'static, Message> {
    let comment = f.lines.saturating_sub(f.code_lines);
    let lines_text = if threshold > 0 && f.lines >= threshold {
        format!("{} *", f.lines)
    } else {
        f.lines.to_string()
    };
    row![
        text(lines_text).size(13).width(Length::Fixed(COL_LINES)),
        text(f.code_lines.to_string()).size(13).width(Length::Fixed(COL_CODE)),
        text(comment.to_string()).size(13).width(Length::Fixed(COL_COMMENT)),
        text(humansize::format_size(f.bytes, humansize::DECIMAL))
            .size(13)
            .width(Length::Fixed(COL_BYTES)),
        text(f.path.display().to_string()).size(13),
    ]
    .spacing(4)
    .padding([2, 8])
    .into()
}
