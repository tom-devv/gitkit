use std::time::Instant;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Text},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::tui::GITKIT_ASCII;

pub struct LoadingWidget {}

pub struct LoadingState {
    start_time: Instant,
}

impl LoadingState {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

impl Default for LoadingWidget {
    fn default() -> Self {
        Self {}
    }
}

impl StatefulWidget for LoadingWidget {
    type State = LoadingState;

    // clean this up
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let elapsed_millis = state.start_time.elapsed().as_millis();

        let current_step = (elapsed_millis / 300) as usize;
        let dot_count = current_step % 4;

        let dots = ".".repeat(dot_count);
        let spaces = " ".repeat(3 - dot_count);

        let mut text = Text::from(GITKIT_ASCII);

        text.lines.push(Line::from(""));
        text.lines
            .push(Line::from(format!("Loading {}{}", dots, spaces)));

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default());

        paragraph.render(area, buf);
    }
}
