//! One module per screen. Each view is a free function
//! `fn view(app: &App) -> Element<'_, Message>` so we can keep all state
//! ownership in `App` and avoid per-screen substates.

pub mod about;
pub mod dashboard;
pub mod duplicates;
pub mod history;
pub mod largest;
pub mod line_count;
pub mod reports;
pub mod scan;
pub mod scheduler;
pub mod settings;
pub mod sidebar;
pub mod treemap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Dashboard,
    Scan,
    Treemap,
    Largest,
    Duplicates,
    LineCount,
    History,
    Reports,
    Scheduler,
    Settings,
    About,
}

impl Screen {
    pub const ALL: &'static [Screen] = &[
        Screen::Dashboard,
        Screen::Scan,
        Screen::Treemap,
        Screen::Largest,
        Screen::Duplicates,
        Screen::LineCount,
        Screen::History,
        Screen::Reports,
        Screen::Scheduler,
        Screen::Settings,
        Screen::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Scan => "Scan",
            Screen::Treemap => "Treemap",
            Screen::Largest => "Largest files",
            Screen::Duplicates => "Duplicates",
            Screen::LineCount => "Line count",
            Screen::History => "History",
            Screen::Reports => "Reports",
            Screen::Scheduler => "Scheduler",
            Screen::Settings => "Settings",
            Screen::About => "About",
        }
    }
}
