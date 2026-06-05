use std::collections::HashMap;
use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent};
use git2::Patch;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Color::Gray;
use ratatui::style::palette::material::{GRAY, WHITE};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{
    Block, LineGauge, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use ratatui::{Frame, symbols};

use crate::error::Result;
use crate::tui::ACCENT;
use crate::{git::kit::KitRepo, tui::Renderable};

#[derive(Default)]
pub struct SiloData {
    pub files: Vec<FileSilo>,
}

#[derive(Default, Debug)]
pub struct FileSilo {
    pub file: String,
    pub gatekeeper: String,
    pub contributors: u16,
    pub risk: u8,
    pub total_churn: usize,
    pub author_churn: HashMap<String, usize>,
}

impl SiloData {
    pub fn new(repo: &KitRepo) -> Self {
        SiloData::get_churn(repo).unwrap_or_default()
    }

    pub fn get_churn(repo: &KitRepo) -> Result<Self> {
        let raw_churn = repo.iter_all_diffs(None)?.fold(
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

        let head_tree = repo.inner.head()?.peel_to_tree()?;
        let mut files: Vec<FileSilo> = raw_churn
            .into_iter()
            .filter_map(|(file, author_churn)| {
                if head_tree.get_path(Path::new(&file)).is_err() {
                    return None;
                }

                let total_churn: usize = author_churn.values().sum();
                let contributors = author_churn.len() as u16;

                let (gatekeeper, top_churn) = author_churn
                    .iter()
                    .max_by_key(|&(_, &churn)| churn)
                    .map(|(author, &churn)| (author.clone(), churn))
                    .unwrap_or_else(|| ("Unknown".to_string(), 0));

                let risk = if total_churn > 0 {
                    ((top_churn as f64 / total_churn as f64) * 100.0).round() as u8
                } else {
                    0
                };

                Some(FileSilo {
                    file,
                    gatekeeper,
                    contributors,
                    risk,
                    total_churn,
                    author_churn,
                })
            })
            .collect();
        files.sort_by(|a, b| b.risk.cmp(&a.risk).then(b.total_churn.cmp(&a.total_churn)));

        Ok(Self { files })
    }
}

pub struct SiloPage {
    data: SiloData,
    scroll_state: ScrollbarState,
    table_state: TableState,
    selected_index: usize,
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

impl SiloPage {
    pub fn new(data: SiloData) -> Self {
        let churn_size = &data.files.len();
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
            _ => {}
        };
    }

    pub fn next(&mut self) {
        if !self.data.files.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.data.files.len();
            self.table_state.select(Some(self.selected_index));
            self.scroll_state = self.scroll_state.position(self.selected_index);
        }
    }

    pub fn prev(&mut self) {
        if !self.data.files.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.data.files.len() - 1;
            } else {
                self.selected_index -= 1;
            }

            self.table_state.select(Some(self.selected_index));
            self.scroll_state = self.scroll_state.position(self.selected_index);
        }
    }

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
            &mut self.scroll_state.position(self.selected_index),
        );
    }

    pub fn render_churn_table(&mut self, frame: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .data
            .files
            .iter()
            .map(|churn| {
                let ratio = churn.risk as f64 / 100.0;
                let bar = SiloPage::generate_silo_bar(ratio, 20); // TODO change fixed width 20
                Row::new(vec![
                    format!("{}", churn.file).fg(WHITE),
                    format!("{}", churn.gatekeeper).fg(WHITE),
                    format!("{}", churn.contributors).fg(WHITE),
                    format!("{} {}%", bar, churn.risk).into(),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(50),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Min(0),
        ];
        let table = Table::new(rows, widths)
            .header(Row::new(vec![
                "PATH".bold(),
                "GATEKEEPER".bold(),
                "CONTRIBUTORS".bold(),
                "SILO RISK".bold(),
            ]))
            .block(
                Block::bordered()
                    .title("Silos")
                    .title_alignment(Alignment::Left),
            )
            .row_highlight_style(ACCENT)
            .highlight_symbol("> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);

        self.render_scrollbar(frame, area);
    }

    pub fn render_churn_info(&self, frame: &mut Frame, area: Rect) {
        let chunks =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

        let left = chunks[0];
        let right = chunks[1];

        self.render_foo(frame, left);

        let block = Block::bordered();

        frame.render_widget(block, right);
    }

    pub fn render_foo(&self, frame: &mut Frame, area: Rect) {
        if let Some(curr_churn) = self.data.files.get(self.selected_index) {
            let block = Block::bordered().title(format!("{}", curr_churn.file));

            frame.render_widget(block, area);
        };
    }

    pub fn generate_silo_bar(percentage: f64, width: usize) -> String {
        let filled = ((percentage) * width as f64).round() as usize;
        let empty = width.saturating_sub(filled);

        let filled_blocks = "█".repeat(filled);
        let empty_blocks = "░".repeat(empty);
        format!("[{}{}]", filled_blocks, empty_blocks)
    }
}
