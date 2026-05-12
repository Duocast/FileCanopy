use iced::widget::{button, column, pick_list, row, scrollable, text};
use iced::{Element, Length};

use crate::dedup::DedupStrategy;
use crate::ui::app::App;
use crate::ui::message::Message;

const STRATEGIES: &[DedupStrategy] = &[
    DedupStrategy::DryRun,
    DedupStrategy::Delete,
    DedupStrategy::Hardlink,
    DedupStrategy::Symlink,
];

pub fn view(app: &App) -> Element<'_, Message> {
    let find_btn = {
        let mut b = button(text("Find duplicates"));
        if app.last_scan.is_some() {
            b = b.on_press(Message::FindDuplicates);
        }
        b
    };

    let apply_btn = {
        let mut b = button(text("Apply"));
        if app.duplicates.is_some() {
            b = b.on_press(Message::ApplyDedup);
        }
        b
    };

    let controls = row![
        find_btn,
        pick_list(
            STRATEGIES.iter().copied().map(StrategyChoice).collect::<Vec<_>>(),
            Some(StrategyChoice(app.dedup_strategy)),
            |c| Message::DedupStrategyChanged(c.0),
        ),
        apply_btn,
    ]
    .spacing(8);

    let body: Element<'_, _> = match app.duplicates.as_ref() {
        Some(report) => {
            let mut col = column![text(format!(
                "{} duplicate groups · reclaimable {}",
                report.groups.len(),
                humansize::format_size(report.reclaimable_bytes, humansize::DECIMAL),
            ))]
            .spacing(6);
            for group in &report.groups {
                col = col.push(text(format!(
                    "{} × {} — {}",
                    group.paths.len(),
                    humansize::format_size(group.size, humansize::DECIMAL),
                    group.fingerprint,
                )));
                for p in &group.paths {
                    col = col.push(text(format!("  {}", p.display())).size(12));
                }
            }
            scrollable(col).height(Length::Fill).into()
        }
        None => text("Find duplicates over the most recent scan.").into(),
    };

    column![text("Duplicates").size(24), controls, body].spacing(12).into()
}

// Wrapper so we can implement `Display` for the pick_list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StrategyChoice(DedupStrategy);

impl std::fmt::Display for StrategyChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                DedupStrategy::DryRun => "Dry run",
                DedupStrategy::Delete => "Delete duplicates",
                DedupStrategy::Hardlink => "Replace with hardlinks",
                DedupStrategy::Symlink => "Replace with symlinks",
            }
        )
    }
}

