use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use git2::Patch;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Stylize;
use ratatui::style::palette::material::WHITE;
use ratatui::widgets::{
    Block, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
};

use crate::error::Result;
use crate::tui::ACCENT;
use crate::{git::kit::KitRepo, tui::Renderable};

pub struct SiloData {
    churn: Churn,
}

pub type Churn = Vec<(String, HashMap<String, usize>)>;

impl SiloData {
    pub fn new(repo: &KitRepo) -> Self {
        let churn = SiloData::get_churn(repo).unwrap_or_default();

        Self { churn }
    }

    pub fn get_churn(repo: &KitRepo) -> Result<Churn> {
        let churn = repo.iter_all_diffs(None)?.fold(
            HashMap::new(),
            |mut acc_map: HashMap<String, HashMap<String, usize>>, (commit, diff)| {
                let author_name = commit.author().name().unwrap_or("Unknown").to_string();

                // TODO calculate all deltas in parallel
                for i in 0..diff.deltas().len() / 10 {
                    if let Ok(Some(patch)) = Patch::from_diff(&diff, i) {
                        let delta = patch.delta();
                        if let Some(path) = delta.new_file().path() {
                            let file_path = path.to_string_lossy().to_string();

                            if let Ok((insertions, deletions, _)) = patch.line_stats() {
                                let file_churn = insertions + deletions;

                                *acc_map
                                    .entry(file_path)
                                    .or_default()
                                    .entry(author_name.clone())
                                    .or_default() += file_churn;
                            }
                        }
                    }
                }

                acc_map
            },
        );
        let mut churn: Churn = churn.into_iter().collect();
        churn.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        Ok(churn)
    }
}

pub struct SiloPage {
    data: SiloData,
    scroll_state: ScrollbarState,
    table_state: TableState,
    selected_index: usize,
}

impl SiloPage {
    pub fn new(data: SiloData) -> Self {
        let churn_size = &data.churn.len();
        let scroll_state = ScrollbarState::new(churn_size.clone()).position(0);
        let table_state = TableState::default().with_selected(0);
        Self {
            data,
            scroll_state,
            table_state,
            selected_index: 0,
        }
    }

    pub fn handle_key(&mut self, key_event: KeyEvent, repo: &KitRepo) {
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.prev(),
            // KeyCode::Enter => self.select(repo),
            // KeyCode::Esc | KeyCode::Backspace => self.unselect(),
            _ => {}
        };
    }

    pub fn next(&mut self) {
        if !self.data.churn.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.data.churn.len();
            self.table_state.select(Some(self.selected_index));
            self.scroll_state = self.scroll_state.position(self.selected_index);
        }
    }

    pub fn prev(&mut self) {
        if !self.data.churn.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.data.churn.len() - 1;
            } else {
                self.selected_index -= 1;
            }

            self.table_state.select(Some(self.selected_index));
            self.scroll_state = self.scroll_state.position(self.selected_index);
        }
    }

    // scrollbar needs fixing
    pub fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        self.scroll_state = self
            .scroll_state
            .viewport_content_length(area.height as usize);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scroll_state.position(self.selected_index + 5),
        );
    }

    pub fn render_churn_table(&mut self, frame: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .data
            .churn
            .iter()
            .map(|churn| Row::new(vec![format!("{}", churn.0).fg(WHITE)]))
            .collect();

        let widths = [Constraint::Percentage(50), Constraint::Percentage(30)];

        let table = Table::new(rows, widths)
            .header(Row::new(vec!["path".bold()]))
            .block(
                Block::bordered()
                    .title("Churn Table")
                    .title_alignment(Alignment::Center),
            )
            .row_highlight_style(ACCENT)
            .highlight_symbol("> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);

        self.render_scrollbar(frame, area);
    }

    pub fn render_churn_info(&self, frame: &mut Frame, area: Rect) {
        if let Some(curr_churn) = self.data.churn.get(self.selected_index) {
            let block = Block::bordered().title(format!("{}", curr_churn.0));
            frame.render_widget(block, area);
        } else {
        }
    }
}

impl Renderable for SiloPage {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical(vec![
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Percentage(50),
        ])
        .split(area);

        self.render_churn_table(frame, chunks[0]);
        self.render_churn_info(frame, chunks[2]);
    }
}
