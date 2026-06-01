use ratatui::{
    layout::{Constraint, HorizontalAlignment::Center, Layout},
    style::{Stylize, palette::material::WHITE},
    text::Text,
};

use crate::tui::{ACCENT_TEXT, GITKIT_ASCII, Renderable};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum Page {
    #[default]
    Home = 0,
    Overview = 1, // Numbering is for easy recogonition of order on the top bar
    Cadence = 2,
    Todo = 3,
}

impl Page {
    pub const ALL: [Page; 4] = [Page::Home, Page::Overview, Page::Cadence, Page::Todo];

    pub fn to_str(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Overview => "Overview",
            Page::Cadence => "Cadence",
            Page::Todo => "Todo",
        }
    }

    pub fn size() -> usize {
        Self::ALL.len()
    }

    pub fn next(&self) -> Page {
        match &self {
            Page::Home => Page::Overview,
            Page::Overview => Page::Cadence,
            Page::Cadence => Page::Todo,
            Page::Todo => Page::Home,
        }
    }
}

pub struct HomePage {
    pub repo_name: String, // maybe use remote with clever url parsing for git repos?
}

impl HomePage {
    pub fn new() -> Self {
        HomePage { repo_name: todo!() }
    }
}

impl Renderable for HomePage {
    fn render(&mut self, frame: &mut ratatui::prelude::Frame, area: ratatui::prelude::Rect) {
        let header_height = GITKIT_ASCII.lines().count() as u16;

        let subtext_height = 3;
        let chunks = Layout::vertical([
            Constraint::Length(header_height),
            Constraint::Length(1), // padding
            Constraint::Length(subtext_height),
            Constraint::Min(0), // rest of page
        ])
        .split(area);

        let header = Text::from(GITKIT_ASCII).alignment(Center).style(WHITE);
        let sub_text = Text::from(format!(
            "gitkit version {}\n made by {} \nrepo: {}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_REPOSITORY")
        ))
        .style(ACCENT_TEXT)
        .italic()
        .alignment(Center);

        // other stuff? maybe about the repo

        frame.render_widget(header, chunks[0]);
        frame.render_widget(sub_text, chunks[2]);
    }
}
