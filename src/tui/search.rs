use crossterm::event::{Event, KeyCode, KeyEvent};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::tui::{
    Searchable,
    pages::Page::{self},
    state::{Mode, TuiState},
};

#[derive(Default)]
pub struct Search {
    pub input: Input,
    pub prev_search: Option<String>, //todo remove? could be used to display the "current search" after closing modal
}

impl Search {
    pub fn clear(&mut self) {
        self.input.reset();
        self.prev_search = None;
    }

    pub fn handle_event(state: &mut TuiState, key: &KeyEvent) {
        // unique actions
        match key.code {
            KeyCode::Esc => {
                state.mode = Mode::Normal;
                return;
            }
            KeyCode::Enter => {
                state.mode = Mode::Normal;

                let query = state.search.input.value().to_string();
                state.search.input.reset();
                state.search.prev_search = Some(query.clone());

                Self::trigger_searched(state, &query);
                return;
            }
            _ => {}
        }

        // any key press
        state.search.input.handle_event(&Event::Key(*key));

        let current_query = state.search.input.value().to_string();
        Self::trigger_update(state, &current_query);
    }

    fn search_event(state: &mut TuiState, mut action: impl FnMut(&mut dyn Searchable)) {
        match state.active_page {
            Page::Silo => {
                if let Some(silo) = state.silo.as_mut() {
                    action(silo);
                }
            }
            Page::Cadence => {
                if let Some(cadence) = state.cadence.as_mut() {
                    action(cadence);
                }
            }
            Page::Home => {}
        }
    }

    pub fn trigger_searched(state: &mut TuiState, query: &str) {
        Self::search_event(state, |page| page.searched(query));
    }

    pub fn trigger_update(state: &mut TuiState, query: &str) {
        Self::search_event(state, |page| page.update(query));
    }
}
