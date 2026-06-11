use std::collections::{HashMap, HashSet};

use crossterm::event::{KeyEvent, MouseEvent};
use git2::{Patch, TreeWalkMode, TreeWalkResult};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Color::{self};
use ratatui::style::palette::material::WHITE;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use crate::error::Result;
use crate::tui::ACCENT;
use crate::tui::widgets::scroll_table::{ScrollingTable, ScrollingTableState};
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

            // TODO ENSURE NO DIVISION HERE

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
    pub data: SiloData,
    pub scrolling_table_state: ScrollingTableState,
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
        let data_len = data.files.len();
        Self {
            data,
            scrolling_table_state: ScrollingTableState::new(data_len),
        }
    }

    pub fn handle_key(&mut self, key_event: KeyEvent, _repo: &KitRepo) {
        self.scrolling_table_state.handle_scroll(&key_event);
    }

    pub fn handle_mouse(&mut self, mouse_event: MouseEvent) {
        self.scrolling_table_state.handle_mouse(&mouse_event);
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

        frame.render_stateful_widget(
            ScrollingTable::new(table),
            area,
            &mut self.scrolling_table_state,
        );
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
        let silo = match self
            .data
            .files
            .get(self.scrolling_table_state.selected_index)
        {
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
