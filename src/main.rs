#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use filecanopy::ui;

fn main() -> iced::Result {
    filecanopy::telemetry::init();
    ui::run()
}
