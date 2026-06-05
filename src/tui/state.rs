use crossterm::event::{KeyCode, KeyEvent};

use crate::git::kit::KitRepo;
use crate::metrics::cadence::{CadenceData, CadencePage};

use crate::error::Result;
use crate::metrics::silo::{SiloData, SiloPage};
use crate::tui::page::{HomeData, HomePage, Page};

pub struct TuiState<'repo> {
    pub is_quit: bool,
    pub loading: bool,
    pub active_page: Page,
    pub home: HomePage<'repo>,
    pub cadence: CadencePage,
    pub silo: SiloPage,
}

impl<'repo> TuiState<'repo> {
    pub fn new(repo: &'repo KitRepo) -> Result<TuiState<'repo>> {
        let cadence_data = CadenceData::full_report(repo)?;
        let home_data = HomeData::new(repo);
        let silo_data = SiloData::new(repo);
        Ok(TuiState {
            is_quit: false,
            loading: false,
            active_page: Page::default(),
            home: HomePage::new(home_data),
            cadence: CadencePage::new(cadence_data),
            silo: SiloPage::new(silo_data),
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
                Page::Silo => self.silo.handle_key(key, repo),
                Page::Home => {}
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
                    ("(k,⇧)/(j,⇩)", "up/down"),
                    ("⏎", "Select"),
                    ("q", "quit"),
                ]
            }
            Page::Silo => {
                vec![("Tab", "Next"), ("(k,⇧)/(j,⇩)", "up/down"), ("q", "quit")]
            }
        }
    }
}
