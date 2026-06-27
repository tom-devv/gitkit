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

    pub fn trigger_searched(state: &mut TuiState, query: &str) {
        match state.active_page {
            Page::Silo => {
                if let Some(silo) = state.silo.as_mut() {
                    silo.searched(query);
                }
            }
            _ => {}
        }
    }

    pub fn trigger_update(state: &mut TuiState, query: &str) {
        match state.active_page {
            Page::Silo => {
                if let Some(silo) = state.silo.as_mut() {
                    silo.update(query);
                }
            }
            _ => {}
        }
    }
}
