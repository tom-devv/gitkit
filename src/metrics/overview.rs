use crate::tui::Renderable;

pub struct OverviewPage {
    pub data: OverviewData,
}

#[derive(Default)]
pub struct OverviewData {}

impl OverviewPage {
    pub fn new(data: OverviewData) -> Self {
        Self { data }
    }
}

impl Renderable for OverviewPage {
    fn render(&mut self, _frame: &mut ratatui::prelude::Frame, _area: ratatui::prelude::Rect) {}
}
