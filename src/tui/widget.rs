use std::{ops::Deref, time::Instant};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::tui::GITKIT_ASCII;

pub struct LoadingWidget {}

pub struct LoadingState {
    frame_counter: usize,
}

impl LoadingState {
    pub fn new() -> Self {
        Self { frame_counter: 0 }
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
        state.frame_counter += 1;

        let current_step = state.frame_counter / 15;

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
