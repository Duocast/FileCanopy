//! Theme selection. We default to the built-in dark palette; users can swap to
//! light from the Settings screen.

use iced::Theme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeChoice {
    Light,
    Dark,
}

impl Default for ThemeChoice {
    fn default() -> Self {
        Self::Dark
    }
}

impl ThemeChoice {
    pub fn to_iced(self) -> Theme {
        match self {
            ThemeChoice::Light => Theme::Light,
            ThemeChoice::Dark => Theme::Dark,
        }
    }
}
