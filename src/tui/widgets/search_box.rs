use ratatui::{
    layout::Constraint,
    widgets::{Block, BorderType::Thick, Clear, Paragraph, Widget},
};

use crate::tui::{
    ACCENT,
    state::{Mode::Searching, TuiState},
};

pub struct SearchBox<'state> {
    state: &'state TuiState,
}

impl<'state> SearchBox<'state> {
    pub fn new(state: &'state TuiState) -> Self {
        Self { state }
    }
}

impl Widget for SearchBox<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if self.state.mode != Searching {
            return;
        }

        Clear.render(area, buf);

        let search_block = Block::bordered()
            .border_type(Thick)
            .title(" Search ")
            .border_style(ACCENT);

        let paragraph = Paragraph::new(self.state.search.input.value())
            .block(search_block)
            .alignment(ratatui::layout::HorizontalAlignment::Left);

        paragraph.render(area, buf);
    }
}
