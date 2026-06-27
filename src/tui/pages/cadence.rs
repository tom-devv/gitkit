use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Cell, Padding, Paragraph, Row, Table},
};

use crate::{
    git::{kit::KitRepo, metrics::cadence::CadenceData},
    tui::{
        ACCENT, Renderable, Searchable,
        widgets::{
            activity_table::ActivityTable,
            scroll_table::{ScrollingTable, ScrollingTableState},
        },
    },
};

#[derive(Debug)]
pub struct CadencePage {
    pub data: CadenceData,
    pub view_state: ScrollingTableState,
}

impl Renderable for CadencePage {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default().padding(Padding::horizontal(1));

        frame.render_widget(&block, area);

        let inner_area = block.inner(area);

        let left_constraint = Constraint::Percentage(60);
        let right_constraint = Constraint::Percentage(40);
        let middle_spacer = Constraint::Percentage(2);

        let main_columns = Layout::horizontal([left_constraint, middle_spacer, right_constraint])
            .split(inner_area);

        let left_column =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_columns[0]);
        let right_column = main_columns[2];

        self.author_table(frame, left_column[0]);
        self.chart(frame, right_column);
        self.more_info(frame, left_column[1]);

        // show more info frame last, this will draw it on top
    }
}

impl Searchable for CadencePage {
    fn searched(&mut self, value: &str) {
        self.update(value);
    }

    fn update(&mut self, value: &str) {
        let value = value.to_lowercase();
        self.view_state
            .apply_search(&self.data.author_details, |details| {
                details.name.to_lowercase().contains(&value)
            });
    }
}

impl CadencePage {
    pub fn new(data: CadenceData) -> Self {
        let data_len = data.author_details.len();
        Self {
            data,
            view_state: ScrollingTableState::new(data_len),
        }
    }

    fn chart(&self, frame: &mut Frame, area: Rect) {
        // this bar chart is not impacted by searching, todo revisit this?
        let mut authors: Vec<(&String, &f32)> = self
            .data
            .author_details
            .iter()
            .map(|ac| (&ac.name, &ac.commits_per_week))
            .collect();
        authors.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

        let chart_data: Vec<(&str, u64)> = authors
            .into_iter()
            .map(|(author, commits)| (author.as_str(), ((*commits) as f32).round() as u64))
            .filter(|(_, commits)| *commits > 0) // remove non-commiters to save space
            .collect();

        let chart = BarChart::default()
            .block(
                Block::default()
                    .title(" Activity Overview ")
                    .borders(Borders::ALL),
            )
            .data(&chart_data)
            .bar_width(5)
            .bar_gap(2)
            .bar_style(Style::default().fg(ACCENT))
            .value_style(Style::default().fg(Color::Black).bg(ACCENT));

        frame.render_widget(chart, area);
    }

    fn author_table(&mut self, frame: &mut Frame, area: Rect) {
        let widths = [Constraint::Percentage(50), Constraint::Percentage(30)];

        let rows: Vec<Row> = self
            .view_state
            .iter_visible(&self.data.author_details)
            .map(|item| {
                Row::new(vec![
                    Cell::from(item.name.clone())
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from(format!("{:.2} / week", item.commits_per_week))
                        .style(Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect();

        let table = Table::new(rows, widths)
            .header(Row::new(vec!["EMAIL".bold(), "CADENCE".bold()]))
            .block(Block::default().title(" Authors ").borders(Borders::ALL))
            .row_highlight_style(ACCENT)
            .highlight_symbol("> ");

        frame.render_stateful_widget(ScrollingTable::new(table), area, &mut self.view_state);
    }

    pub fn more_info(&self, frame: &mut Frame, area: Rect) {
        let Some(details) = self.view_state.get_selected(&self.data.author_details) else {
            return;
        };

        frame.render_widget(
            Block::bordered()
                .title(format!(" {} ", details.name))
                .title_style(Color::White)
                .title_alignment(Alignment::Center),
            area,
        ); // or left? silo is left but that looks good this wont, area);

        let layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area.inner(Margin {
                horizontal: 1,
                vertical: 2,
            }));

        let key_style = Style::default().fg(Color::White);
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Total Commits: ", key_style),
                Span::raw(format!("{}", details.total_commits)),
            ]),
            Line::from(vec![
                Span::styled("  Commits/Week:  ", key_style),
                Span::raw(format!("{}", details.commits_per_week)),
            ]),
            Line::from(vec![
                Span::styled("  First Commit:  ", key_style),
                Span::raw(format!("{}", details.first_commit)),
            ]),
            Line::from(vec![
                Span::styled("  Repo Share:    ", key_style),
                Span::raw(format!("{:.2}%", details.repo_share)),
            ]),
        ];

        let paragraph = Paragraph::new(text).alignment(Alignment::Left);

        frame.render_widget(paragraph, layout[0]);

        frame.render_widget(ActivityTable::new(&details.all_commits), layout[1]);
    }

    pub fn handle_key(&mut self, key_event: KeyEvent, _repo: &KitRepo) {
        self.view_state.handle_scroll(&key_event);
    }

    pub fn handle_mouse(&mut self, mouse_event: MouseEvent) {
        self.view_state.handle_mouse(&mouse_event);
    }
}
