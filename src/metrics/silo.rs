use crate::tui::Renderable;

pub struct SiloData {}

pub struct SiloPage {}

impl Renderable for SiloPage {
    fn render(&mut self, _frame: &mut ratatui::prelude::Frame, _area: ratatui::prelude::Rect) {}
}
