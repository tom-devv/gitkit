use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize, palette::material::WHITE},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::{
    git::{kit::KitRepo, metrics::silo::SiloData},
    tui::{
        ACCENT, Renderable, Searchable,
        widgets::scroll_table::{ScrollingTable, ScrollingTableState},
    },
};

pub struct SiloPage {
    pub data: SiloData,
    pub scrolling_table_state: ScrollingTableState,
    pub search_filter: Vec<usize>,
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

impl Searchable for SiloPage {
    fn searched(&mut self, value: &str) {
        self.update(value);
    }

    fn update(&mut self, value: &str) {
        let value = value.to_lowercase();
        self.search_filter = self
            .data
            .files
            .iter()
            .enumerate()
            .filter(|(_, file)| file.file.to_lowercase().contains(&value)) // adjust to your actual data struct
            .map(|(index, _)| index)
            .collect();

        let new_len = self.search_filter.len();
        self.scrolling_table_state.data_len = new_len;

        self.scrolling_table_state.selected_index = 0;
    }
}

impl SiloPage {
    pub fn new(data: SiloData) -> Self {
        let data_len = data.files.len();
        let search_filter: Vec<usize> = (0..data_len).collect();
        Self {
            data,
            scrolling_table_state: ScrollingTableState::new(data_len),
            search_filter,
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
            .search_filter
            .iter()
            .map(|&i| &self.data.files[i])
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

        self.render_file_info(frame, left);

        let block = Block::bordered();

        frame.render_widget(block, right);
    }

    pub fn render_file_info(&self, frame: &mut Frame, area: Rect) {
        let silo = match self
            .search_filter
            .get(self.scrolling_table_state.selected_index)
        {
            Some(&i) => &self.data.files[i],
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
