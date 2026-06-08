use std::collections::{HashMap, HashSet};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use git2::{Patch, TreeWalkMode, TreeWalkResult};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Color::{self};
use ratatui::style::palette::material::WHITE;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    TableState,
};

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
        let head_files = Self::get_head_files(repo)?;

        let raw_churn_map = Self::accumulate_churn(repo)?;

        let active_files = Self::process_silos(raw_churn_map, &head_files);

        Ok(Self {
            files: active_files,
        })
    }

    pub fn get_head_files(repo: &KitRepo) -> Result<HashSet<String>> {
        let mut current_files = HashSet::new();
        let head = repo.inner.head()?;
        let head_tree = head.peel_to_tree()?;

        head_tree.walk(TreeWalkMode::PreOrder, |root, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                if let Some(name) = entry.name().ok() {
                    current_files.insert(format!("{}{}", root, name));
                }
            }
            TreeWalkResult::Ok
        })?;

        Ok(current_files)
    }

    pub fn accumulate_churn(repo: &KitRepo) -> Result<HashMap<String, HashMap<String, usize>>> {
        let mut churn_map: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for (commit, diff) in repo.iter_diff_history()? {
            let author_name = commit.email;

            for i in 0..diff.deltas().len() {
                if let Ok(Some(patch)) = Patch::from_diff(&diff, i) {
                    if let Some(path) = patch.delta().new_file().path() {
                        let file_path = path.to_string_lossy().to_string();

                        if let Ok((insertions, deletions, _)) = patch.line_stats() {
                            let churn = insertions + deletions;

                            if churn > 0 {
                                *churn_map
                                    .entry(file_path)
                                    .or_default()
                                    .entry(author_name.clone())
                                    .or_default() += churn;
                            }
                        }
                    }
                }
            }
        }

        Ok(churn_map)
    }

    pub fn process_silos(
        churn_map: HashMap<String, HashMap<String, usize>>,
        head_files: &HashSet<String>,
    ) -> Vec<FileSilo> {
        let mut active_files = Vec::new();

        for (file, author_churn) in churn_map {
            if !head_files.contains(&file) {
                continue;
            }

            let total_churn: usize = author_churn.values().sum();
            let contributors = author_churn.len() as u16;

            let mut gatekeeper = String::from("Unknown");
            let mut top_churn = 0;

            for (author, churn) in &author_churn {
                if *churn > top_churn {
                    top_churn = *churn;
                    gatekeeper = author.clone();
                }
            }

            let risk = if total_churn > 0 {
                ((top_churn as f64 / total_churn as f64) * 100.0).round() as u8
            } else {
                0
            };

            active_files.push(FileSilo {
                file,
                gatekeeper,
                contributors,
                risk,
                total_churn,
                author_churn,
            });
        }

        active_files.sort_by(|a, b| b.risk.cmp(&a.risk).then(b.total_churn.cmp(&a.total_churn)));

        active_files
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

    pub fn handle_key(&mut self, key_event: KeyEvent, _repo: &KitRepo) {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => self.next(1),
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => self.prev(1),

            (KeyCode::Char('g'), KeyModifiers::NONE) => self.top(),
            // G (with caps lock) or G (with shift)
            (KeyCode::Char('G'), _) | (KeyCode::Char('g'), KeyModifiers::SHIFT) => self.bottom(),

            // shift + j/k = 5 skips
            (KeyCode::Char('J'), _) | (KeyCode::Char('j'), KeyModifiers::SHIFT) => self.next(5),
            (KeyCode::Char('K'), _) | (KeyCode::Char('k'), KeyModifiers::SHIFT) => self.prev(5),
            _ => {}
        }
    }

    fn select_index(&mut self, index: usize) {
        self.selected_index = index;
        self.table_state.select(Some(self.selected_index));
        self.scroll_state = self.scroll_state.position(self.selected_index);
    }

    pub fn top(&mut self) {
        if !self.data.files.is_empty() {
            self.select_index(0);
        }
    }

    pub fn bottom(&mut self) {
        if !self.data.files.is_empty() {
            self.select_index(self.data.files.len() - 1);
        }
    }

    pub fn next(&mut self, skip: usize) {
        if !self.data.files.is_empty() {
            self.select_index((self.selected_index + skip) % self.data.files.len());
        }
    }

    pub fn prev(&mut self, skip: usize) {
        if !self.data.files.is_empty() {
            let len = self.data.files.len();
            if self.selected_index < skip {
                self.selected_index = (len + self.selected_index - (skip % len)) % len;
            } else {
                self.selected_index -= skip;
            }

            self.select_index(self.selected_index); // this is kinda wrong but it works
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
                let bar = generate_silo_bar(ratio, 20); // TODO change fixed width 20
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
        let silo = match self.data.files.get(self.selected_index) {
            Some(silo) => silo,
            None => return,
        };

        let mut top_contributors: Vec<(&String, &usize)> = silo.author_churn.iter().collect();
        top_contributors.sort_by(|a, b| b.1.cmp(a.1));

        let mut info_lines = vec![
            Line::from(vec![
                Span::styled("Total File Churn: ", Style::default().fg(Color::White)),
                Span::styled(
                    silo.total_churn.to_string(),
                    Style::default().fg(Color::White),
                ),
                Span::raw(" lines"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Top Contributors:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ];

        for (author, churn) in top_contributors.iter().take(3) {
            let percentage = (**churn as f64 / silo.total_churn as f64) * 100.0;
            info_lines.push(Line::from(format!(
                "  - {}: {} lines ({:.0}%)",
                author, churn, percentage
            )));
        }

        let info_paragraph = Paragraph::new(info_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(silo.file.clone())
                .style(Style::default().fg(Color::Gray)),
        );

        frame.render_widget(info_paragraph, area);
    }
}

fn generate_silo_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let filled_blocks = "█".repeat(filled);
    let empty_blocks = "░".repeat(empty);
    format!("[{}{}]", filled_blocks, empty_blocks)
}
