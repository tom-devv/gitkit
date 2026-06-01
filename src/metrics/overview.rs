

pub struct OverviewPage {
    pub data: OverviewData,
}

#[derive(Default)]
pub struct OverviewData {}

impl OverviewPage {
    pub fn new(data: OverviewData) -> Self {
        Self { data }
    }

    pub fn render(&self, _frame: &mut ratatui::prelude::Frame, _area: ratatui::prelude::Rect) {}
}
