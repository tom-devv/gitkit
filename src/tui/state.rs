use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

use crate::git::kit::KitRepo;
use crate::metrics::cadence::CadencePage;

use crate::error::Result;
use crate::metrics::silo::SiloPage;
use crate::tui::page::{HomePage, Page};
use crate::tui::widgets::loading::LoadingState;
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
    pub fn new(data: DataPayload) -> Result<TuiState> {
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
            Page::Cadence => self.cadence.handle_mouse(mouse),
            Page::Silo => self.silo.handle_mouse(mouse),
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
                    ("j/k/⇅/Scroll", "Move"),
                    ("J/K", "5x"),
                    ("g/G", "Top/Bot"),
                    ("⏎", "Select"),
                    ("q", "quit"),
                ]
            }
            Page::Silo => {
                vec![
                    ("Tab", "Next"),
                    ("j/k/⇅/Scroll", "Move"),
                    ("J/K", "5x"),
                    ("g/G", "Top/Bot"),
                    ("q", "Quit"),
                ]
            }
        }
    }
}
