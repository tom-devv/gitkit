use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, HorizontalAlignment::Center, Layout, Rect},
    style::{Stylize, palette::material::WHITE},
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph, Wrap},
};

use crate::{
    git::metrics::home::HomeData,
    git::{kit::KitRepo, util},
    tui::{ACCENT, ACCENT_TEXT, GITKIT_ASCII, Renderable},
};

pub struct HomePage {
    data: HomeData,
}

impl HomePage {
    pub fn new(data: HomeData) -> Self {
        HomePage { data }
    }

    pub fn handle_key(&mut self, key_event: KeyEvent, _repo: &KitRepo, refresh: &mut bool) {
        match key_event.code {
            KeyCode::Char('r') => *refresh = true,
            _ => {}
        }
    }

    pub fn handle_mouse(&mut self, _mouse_event: MouseEvent) {}

    // TODO make this scrollable
    fn info_box(&self, frame: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            "|<".into(),
            "Repository: ".fg(ACCENT_TEXT).bold(),
            format!("{}", self.data.repo_name).fg(ACCENT),
            ">|".into(),
        ]);
        let general = Block::bordered()
            .title(title)
            .title_alignment(Center)
            .padding(Padding {
                left: 2,
                right: 0,
                top: 1,
                bottom: 0,
            });

        let active_since = util::active_since(&self.data.first_commit);
        let last_activity = util::last_activity(&self.data.last_commit);

        let mut info = vec![
            Line::from(vec![
                "branch: ".fg(ACCENT).bold(),
                format!("{}", self.data.current_branch).fg(WHITE),
            ]),
            Line::from(vec![
                "total commits: ".fg(ACCENT).bold(),
                format!("{}", self.data.total_commits).fg(WHITE),
            ]),
            active_since,
            last_activity,
        ];

        let status = &self.data.status;

        info.push(Line::from("status: ".fg(ACCENT).bold()));
        info.extend(status.tui_print());

        // +2 borders + 1 padding (top) TODO make this better
        // paragraph wrap seems to break the last info lines
        let min_height = info.len() as u16 + (info.len() as u16) / 2 + 3;

        let vertical_chunks =
            Layout::vertical([Constraint::Length(min_height), Constraint::Min(0)]).split(area);

        let info_area = vertical_chunks[0].centered_horizontally(Constraint::Percentage(50));

        let paragraph = Paragraph::new(info)
            .block(general)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, info_area);
    }
}

impl<'repo> Renderable for HomePage {
    fn render(&mut self, frame: &mut ratatui::prelude::Frame, area: ratatui::prelude::Rect) {
        let header_height = GITKIT_ASCII.lines().count() as u16;

        let subtext_height = 3;
        let chunks = Layout::vertical([
            Constraint::Length(header_height),
            Constraint::Length(1), // padding
            Constraint::Length(subtext_height),
            Constraint::Length(2), // padding
            Constraint::Min(0),    // rest of page
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

        frame.render_widget(header, chunks[0]);
        frame.render_widget(sub_text, chunks[2]);
        self.info_box(frame, chunks[4]);
    }
}
