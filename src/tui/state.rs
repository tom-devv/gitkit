use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

use crate::git::kit::KitRepo;


use crate::tui::pages::Page;
use crate::tui::pages::cadence::CadencePage;
use crate::tui::pages::home::HomePage;
use crate::tui::pages::silo::SiloPage;
use crate::tui::widgets::loading::LoadingState;
use crate::worker::DataUpdate;

pub struct TuiState {
    pub is_quit: bool,
    pub loading_state: LoadingState,
    pub refresh: bool,
    pub active_page: Page,
    pub home: Option<HomePage>,
    pub cadence: Option<CadencePage>,
    pub silo: Option<SiloPage>,
}

impl TuiState {
    pub fn new() -> TuiState {
        TuiState {
            is_quit: false,
            loading_state: LoadingState::new(),
            refresh: false,
            active_page: Page::default(),
            home: None,
            cadence: None,
            silo: None,
        }
    }

    pub fn update(&mut self, data: DataUpdate) {
        match data {
            DataUpdate::Home(home_data) => self.home = Some(HomePage::new(home_data)),
            DataUpdate::Cadence(cadence_data) => {
                self.cadence = Some(CadencePage::new(cadence_data))
            }
            DataUpdate::Silo(silo_data) => self.silo = Some(SiloPage::new(silo_data)),
        }
    }

    // there is no global loading as
    // each page can be loading in parallel
    pub fn is_loading(&self) -> bool {
        match self.active_page {
            Page::Home => self.home.is_none(),
            Page::Cadence => self.cadence.is_none(),
            Page::Silo => self.silo.is_none(),
        }
    }

    // create a new tui state, ready to be populated
    // keeps current page the same
    pub fn refresh(&mut self) {
        let current_page = self.active_page.clone();
        *self = TuiState::new();
        self.active_page = current_page;
    }

    pub fn next_tab(&mut self) {
        self.active_page = self.active_page.next();
    }

    pub fn handle_key_event(&mut self, key: KeyEvent, repo: &KitRepo) {
        match key.code {
            KeyCode::Char('q') => self.is_quit = true,
            KeyCode::Char('r') => self.refresh = true,
            KeyCode::Tab => self.next_tab(),
            // this if let is a bit verbose
            _ => match self.active_page {
                Page::Cadence => {
                    if let Some(cadence_page) = &mut self.cadence {
                        cadence_page.handle_key(key, repo);
                    }
                }
                Page::Silo => {
                    if let Some(silo_page) = &mut self.silo {
                        silo_page.handle_key(key, repo);
                    }
                }
                Page::Home => {
                    if let Some(home_page) = &mut self.home {
                        home_page.handle_key(key, repo);
                    }
                }
            },
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent, _repo: &KitRepo) {
        match self.active_page {
            // also verbose?
            Page::Cadence => {
                if let Some(cadence_page) = &mut self.cadence {
                    cadence_page.handle_mouse(mouse);
                }
            }
            Page::Silo => {
                if let Some(silo_page) = &mut self.silo {
                    silo_page.handle_mouse(mouse);
                }
            }
            Page::Home => {
                if let Some(home_page) = &mut self.home {
                    home_page.handle_mouse(mouse);
                }
            }
        }
    }

    pub fn get_binds(&self) -> Vec<(&str, &str)> {
        match self.active_page {
            Page::Home => {
                vec![("Tab", "Next"), ("r", "refresh"), ("q", "quit")]
            }
            Page::Cadence => {
                vec![
                    ("Tab", "Next"),
                    ("j/k/⇅/Scroll", "Move"),
                    ("J/K", "5x"),
                    ("g/G", "Top/Bot"),
                    ("r", "refresh"),
                    ("q", "quit"),
                ]
            }
            Page::Silo => {
                vec![
                    ("Tab", "Next"),
                    ("j/k/⇅/Scroll", "Move"),
                    ("J/K", "5x"),
                    ("g/G", "Top/Bot"),
                    ("r", "refresh"),
                    ("q", "Quit"),
                ]
            }
        }
    }
}
