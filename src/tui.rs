use ratatui::style::Color;

pub mod page;
pub mod state;
pub mod ui;

pub const GRAY_BORDER_COLOR: ratatui::prelude::Color = Color::Rgb(105, 103, 97);
pub const PRIMARY: ratatui::prelude::Color = Color::White;
pub const ACCENT: ratatui::prelude::Color = Color::Rgb(220, 138, 120);
pub const ACCENT_TEXT: ratatui::prelude::Color = Color::Rgb(204, 208, 218);
