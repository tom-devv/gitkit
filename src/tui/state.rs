use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

use crate::git::kit::KitRepo;
use crate::metrics::cadence::{CadenceData, CadencePage};

use crate::error::Result;
use crate::metrics::silo::{SiloData, SiloPage};
use crate::tui::page::{HomeData, HomePage, Page};
use crate::tui::widget::LoadingState;
use crate::worker::DataPayload;

pub struct TuiState {
    pub is_quit: bool,
    pub loading: bool,
    pub loading_state: LoadingState,
    pub refresh: bool,
    pub active_page: Page,
    pub home: HomePage,
    pub cadence: CadencePage,
    pub silo: SiloPage,
}

impl TuiState {
    pub fn new(repo: &KitRepo) -> Result<TuiState> {
        // putting in struct so i dont forget new pages from the payload struct
        let data = DataPayload {
            cadence_data: CadenceData::new(repo),
            home_data: HomeData::new(repo),
            silo_data: SiloData::new(repo),
        };

        Ok(TuiState {
            is_quit: false,
            loading: false,
            loading_state: LoadingState::new(),
            refresh: false,
            active_page: Page::default(),
            home: HomePage::new(data.home_data),
            cadence: CadencePage::new(data.cadence_data),
            silo: SiloPage::new(data.silo_data),
        })
    }

    pub fn refresh(&mut self, payload: DataPayload) {
        self.home = HomePage::new(payload.home_data);
        self.cadence = CadencePage::new(payload.cadence_data);
        self.silo = SiloPage::new(payload.silo_data);

        self.loading = false;
        self.refresh = false;
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
                Page::Home => self.home.handle_key(key, repo, &mut self.refresh),
            },
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent, _repo: &KitRepo) {
        match self.active_page {
            Page::Home => self.home.handle_mouse(mouse),
            Page::Cadence => {}
            Page::Silo => {}
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
                vec![
                    ("Tab", "Next"),
                    ("(k,⇧)/(j,⇩)", "up/down"),
                    ("(SHIFT + k)/(shift + j)", "5 (up/down)"),
                    ("g/G", "top/bottom"),
                    ("q", "quit"),
                ]
            }
        }
    }
}
