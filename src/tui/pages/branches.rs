use chrono::Utc;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::material::WHITE},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::{
    git::{
        kit::KitRepo,
        metrics::branches::{BranchData, BranchInfo, BranchKind, BranchStatus, STALE_AFTER_DAYS},
    },
    tui::{
        ACCENT, Renderable, Searchable,
        widgets::scroll_table::{ScrollingTable, ScrollingTableState},
    },
};

pub struct BranchesPage {
    pub data: BranchData,
    pub view_state: ScrollingTableState,
}

impl Renderable for BranchesPage {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical(vec![
            Constraint::Percentage(60),
            Constraint::Length(1),
            Constraint::Percentage(40),
        ])
        .split(area);

        self.render_branch_table(frame, chunks[0]);

        let bottom =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[2]);

        self.render_branch_info(frame, bottom[0]);
        self.render_summary(frame, bottom[1]);
    }
}

impl Searchable for BranchesPage {
    fn searched(&mut self, value: &str) {
        self.update(value);
    }

    fn update(&mut self, value: &str) {
        let value = value.to_lowercase();
        self.view_state.apply_search(&self.data.branches, |branch| {
            branch.name.to_lowercase().contains(&value) || branch.status().as_str().contains(&value)
        });
    }
}

impl BranchesPage {
    pub fn new(data: BranchData) -> Self {
        let data_len = data.branches.len();
        Self {
            data,
            view_state: ScrollingTableState::new(data_len),
        }
    }

    pub fn handle_key(&mut self, key_event: KeyEvent, _repo: &KitRepo) {
        self.view_state.handle_scroll(&key_event);
    }

    pub fn handle_mouse(&mut self, mouse_event: MouseEvent) {
        self.view_state.handle_mouse(&mouse_event);
    }

    fn render_branch_table(&mut self, frame: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .view_state
            .iter_visible(&self.data.branches)
            .map(|branch| {
                let status = branch.status();
                let name = if branch.is_head {
                    format!("* {}", branch.name)
                } else {
                    branch.name.clone()
                };
                Row::new(vec![
                    name.fg(WHITE),
                    status.as_str().fg(status_color(status)),
                    format!("↑{} ↓{}", branch.ahead, branch.behind).fg(WHITE),
                    commit_age(branch).fg(WHITE),
                    branch.last_commit.email.clone().fg(WHITE),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(40),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Min(0),
        ];

        let title = match &self.data.base {
            Some(base) => format!("Branches (vs {})", base),
            None => "Branches".to_string(),
        };

        let table = Table::new(rows, widths)
            .header(Row::new(vec![
                "BRANCH".bold(),
                "STATUS".bold(),
                "AHEAD/BEHIND".bold(),
                "LAST".bold(),
                "AUTHOR".bold(),
            ]))
            .block(
                Block::bordered()
                    .title(title)
                    .title_alignment(Alignment::Left),
            )
            .row_highlight_style(ACCENT)
            .highlight_symbol("> ");

        frame.render_stateful_widget(ScrollingTable::new(table), area, &mut self.view_state);
    }

    fn render_branch_info(&self, frame: &mut Frame, area: Rect) {
        let Some(branch) = self.view_state.get_selected(&self.data.branches) else {
            frame.render_widget(Block::bordered(), area);
            return;
        };

        let key_style = Style::default().fg(Color::White);
        let base = self.base();
        let kind = match branch.kind {
            BranchKind::Local => "local",
            BranchKind::Remote => "remote",
        };
        let upstream = match (&branch.upstream, branch.upstream_gone) {
            (Some(upstream), true) => format!("{} (gone)", upstream),
            (Some(upstream), false) => upstream.clone(),
            (None, _) => "none".to_string(),
        };
        let mut hash = branch.last_commit.id.clone();
        hash.truncate(7);

        let status = match branch.status() {
            BranchStatus::Merged if branch.squash_merged => "merged (squash)",
            status => status.as_str(),
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Status:      ", key_style),
                Span::styled(status, Style::default().fg(status_color(branch.status()))),
            ]),
            Line::from(vec![
                Span::styled("Kind:        ", key_style),
                Span::raw(kind),
            ]),
            Line::from(vec![
                Span::styled("Upstream:    ", key_style),
                Span::raw(upstream),
            ]),
            Line::from(vec![
                Span::styled("Ahead:       ", key_style),
                Span::raw(format!("{} commits not in {}", branch.ahead, base)),
            ]),
            Line::from(vec![
                Span::styled("Behind:      ", key_style),
                Span::raw(format!("{} commits behind {}", branch.behind, base)),
            ]),
            Line::from(vec![
                Span::styled("Last commit: ", key_style),
                Span::raw(format!(
                    "({}) {} by {}",
                    hash,
                    branch
                        .last_commit
                        .date
                        .map(|d| d.format("%Y/%m/%d").to_string())
                        .unwrap_or_default(),
                    branch.last_commit.email
                )),
            ]),
        ];

        let info = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(branch.name.clone())
                .style(Style::default().fg(Color::Gray)),
        );

        frame.render_widget(info, area);
    }

    fn render_summary(&self, frame: &mut Frame, area: Rect) {
        let count = |status| {
            self.data
                .branches
                .iter()
                .filter(|b| b.status() == status)
                .count()
        };

        let mut lines = vec![Line::from(vec![
            Span::styled("Total branches: ", Style::default().fg(Color::White)),
            Span::raw(self.data.branches.len().to_string()),
        ])];
        lines.push(Line::from(""));

        let explanations = [
            (
                BranchStatus::Merged,
                format!("already in {}, safe to delete", self.base()),
            ),
            (BranchStatus::Gone, "upstream was deleted".to_string()),
            (
                BranchStatus::Stale,
                format!("unmerged, no commits in {} days", STALE_AFTER_DAYS),
            ),
            (
                BranchStatus::Active,
                "unmerged, recently updated".to_string(),
            ),
        ];
        for (status, explanation) in explanations {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<7} {:>4}", status.as_str(), count(status)),
                    Style::default()
                        .fg(status_color(status))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("  {}", explanation)),
            ]));
        }

        let summary = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Hygiene")
                .style(Style::default().fg(Color::Gray)),
        );

        frame.render_widget(summary, area);
    }

    fn base(&self) -> &str {
        self.data.base.as_deref().unwrap_or("base")
    }
}

fn status_color(status: BranchStatus) -> Color {
    match status {
        BranchStatus::Merged => Color::Green,
        BranchStatus::Gone => Color::Red,
        BranchStatus::Stale => Color::Yellow,
        BranchStatus::Active => Color::White,
    }
}

// compact age of the branch tip, e.g. "3d ago"
fn commit_age(branch: &BranchInfo) -> String {
    let Some(date) = branch.last_commit.date else {
        return "?".to_string();
    };
    let days = Utc::now().signed_duration_since(date).num_days().max(0);
    match days {
        0 => "today".to_string(),
        1..=13 => format!("{}d ago", days),
        14..=59 => format!("{}w ago", days / 7),
        60..=729 => format!("{}mo ago", days / 30),
        _ => format!("{}y ago", days / 365),
    }
}
