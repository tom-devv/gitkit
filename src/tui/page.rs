
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, HorizontalAlignment::Center, Layout, Rect},
    style::{Stylize, palette::material::WHITE},
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph},
};

use crate::{
    git::{kit::KitRepo, status::KitStatus},
    tui::{ACCENT, ACCENT_TEXT, GITKIT_ASCII, Renderable},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum Page {
    #[default]
    Home = 0,
    Cadence = 1,
    Todo = 2,
}

impl Page {
    pub const ALL: [Page; 3] = [Page::Home, Page::Cadence, Page::Todo];

    pub fn to_str(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Cadence => "Cadence",
            Page::Todo => "Todo",
        }
    }

    pub fn size() -> usize {
        Self::ALL.len()
    }

    pub fn next(&self) -> Page {
        match &self {
            Page::Home => Page::Cadence,
            Page::Cadence => Page::Todo,
            Page::Todo => Page::Home,
        }
    }
}

pub struct HomeData {
    pub repo_name: String, // directory name
    pub current_branch: String,
    pub total_commits: u32,
    pub status: KitStatus,
}

impl HomeData {
    pub fn new(repo: &KitRepo) -> Self {
        let workdir = repo.inner.workdir().unwrap_or_else(|| repo.inner.path());

        let repo_name = workdir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unkown workdir".to_string());
        let current_branch = repo
            .current_branch()
            .unwrap_or_else(|_| "not found".to_owned());

        let total_commits: u32 = repo
            .iter_commits()
            .map_or(0, |iter| iter.count())
            .try_into()
            .unwrap_or(u32::MAX);

        let status = repo.get_status();

        HomeData {
            repo_name,
            current_branch,
            total_commits,
            status,
        }
    }
}

pub struct HomePage {
    data: HomeData,
}

impl HomePage {
    pub fn new(data: HomeData) -> Self {
        HomePage { data }
    }
}

impl HomePage {
    // TODO make this scrollable
    fn info_box(&self, frame: &mut Frame, area: Rect) {
        let info_area = area.centered(Constraint::Percentage(50), Constraint::Percentage(50));
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

        let mut info = vec![
            Line::from(vec![
                "branch: ".fg(ACCENT).bold(),
                format!("{}", self.data.current_branch).fg(WHITE),
            ]),
            Line::from(vec![
                "total commits: ".fg(ACCENT).bold(),
                format!("{}", self.data.total_commits).fg(WHITE),
            ]),
            Line::from("status: ".fg(ACCENT).bold()),
        ];

        let status = &self.data.status;

        info.extend(status.tui_print());

        let paragraph = Paragraph::new(info)
            .block(general)
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, info_area);
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

        frame.render_widget(header, chunks[0]);
        frame.render_widget(sub_text, chunks[2]);
        self.info_box(frame, chunks[3]);
    }
}
