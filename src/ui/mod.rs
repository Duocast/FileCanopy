//! iced 0.14 GUI for FileCanopy.
//!
//! Architecture (Elm-style):
//! - [`app::App`]      holds the entire application state.
//! - [`message::Message`] is every event the UI can emit.
//! - [`app::App::update`] mutates state in response to messages.
//! - [`app::App::view`]   renders the current screen.
//! - [`views`]         is one module per screen (sidebar entry).
//! - [`widgets`]       holds reusable / custom widgets, including the treemap canvas.

pub mod app;
pub mod message;
pub mod tasks;
pub mod theme;
pub mod views;
pub mod widgets;

use app::App;

/// Launch the GUI. Blocks the calling thread until the window is closed.
pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window_size((1280.0, 800.0))
        .run()
}
