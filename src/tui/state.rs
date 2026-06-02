use crossterm::event::{KeyCode, KeyEvent};

use crate::git::kit::KitRepo;
use crate::metrics::cadence::{CadenceData, CadencePage};

use crate::error::Result;
use crate::tui::page::{HomeData, HomePage, Page};

pub struct TuiState {
    pub is_quit: bool,
    pub loading: bool,
    pub active_page: Page,
    pub home: HomePage,
    pub cadence: CadencePage,
}

impl TuiState {
    //By default new stats will be loading
    pub fn new(repo: &KitRepo) -> Result<TuiState> {
        let cadence_data = CadenceData::full_report(repo)?;
        let home_data = HomeData::new(repo);
        Ok(TuiState {
            is_quit: false,
            loading: false,
            active_page: Page::default(),
            home: HomePage::new(home_data),
            cadence: CadencePage::new(cadence_data),
        })
    }

    pub fn next_tab(&mut self) {
        self.active_page = self.active_page.next();
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
            Page::Home => {
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
