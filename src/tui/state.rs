use crossterm::event::{KeyCode, KeyEvent};

use crate::git::kit::KitRepo;
use crate::metrics::cadence::{CadenceData, CadencePage};
use crate::metrics::overview::{OverviewData, OverviewPage};

use crate::error::Result;
use crate::tui::page::Page;

pub struct TuiState {
    pub is_quit: bool,
    pub loading: bool,
    pub active_page: Page,
    pub overview: OverviewPage,
    pub cadence: CadencePage,
}

impl TuiState {
    //By default new stats will be loading
    pub fn new(repo: &KitRepo) -> Result<TuiState> {
        let cadence_data = CadenceData::full_report(repo)?;
        let overview_data = OverviewData::default();
        Ok(TuiState {
            is_quit: false,
            loading: false,
            active_page: Page::default(),
            overview: OverviewPage::new(overview_data),
            cadence: CadencePage::new(cadence_data),
        })
    }

    pub fn next_tab(&mut self) {
        let next_page = match self.active_page {
            Page::Overview => Page::Cadence,
            Page::Cadence => Page::Todo,
            Page::Todo => Page::Overview,
        };
        self.active_page = next_page;
    }

    pub fn handle_key_event(&mut self, key: KeyEvent, repo: &KitRepo) {
        match key.code {
            KeyCode::Char('q') => self.is_quit = true, // todo add ctrl + c as quit
            KeyCode::Tab => self.next_tab(),

            _ => match self.active_page {
                Page::Cadence => self.cadence.handle_key(key, repo),
                // Page::Overview => self.overview.handle_key(key.code),
                Page::Todo => {}
                _ => {}
            },
        }
    }

    pub fn get_binds(&self) -> Vec<(&str, &str)> {
        match self.active_page {
            Page::Overview => {
                vec![("Tab", "Next"), ("q", "quit")]
            }
            Page::Cadence => {
                vec![
                    ("Tab", "Next"),
                    ("(j,⇧)/(k,⇩)", "down/up"),
                    ("⏎", "Select"),
                    ("q", "quit"),
                ]
            }

            _ => vec![("Tab", "Next"), ("q", "quit")],
        }
    }
}
