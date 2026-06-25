use std::time::Instant;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Style, Stylize, palette::material::WHITE},
    text::{Line, Text},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::tui::GITKIT_ASCII;

pub struct LoadingWidget {
    dots: bool,
    text: &'static str,
}

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
        Self {
            dots: true,
            text: "Loading",
        }
    }
}

impl LoadingWidget {
    pub fn text(mut self, text: &'static str) -> Self {
        self.text = text;
        self
    }
    pub fn dots(mut self, dots: bool) -> Self {
        self.dots = dots;
        self
    }
}

impl StatefulWidget for LoadingWidget {
    type State = LoadingState;

    // clean this up
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let elapsed_millis = state.start_time.elapsed().as_millis();

        let current_step = (elapsed_millis / 300) as usize;
        let dot_count = current_step % 4;

        let dots = "■ ".repeat(dot_count);
        let spaces = "  ".repeat(3 - dot_count);

        let mut text = Text::from(GITKIT_ASCII).fg(WHITE);

        let mut sub_text = Line::from(format!("{}", self.text));
        if self.dots {
            sub_text.push_span(format!(" {}{}", dots, spaces));
        }

        text.lines.push(Line::from(""));
        text.lines.push(sub_text);

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default());

        paragraph.render(area, buf);
    }
}
