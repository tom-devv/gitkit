use chrono::{DateTime, TimeDelta, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use git2::Commit;

use crate::git::kit::KitRepo;
use crate::{error::Result, tui::ACCENT};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        BarChart, Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Table,
        TableState,
    },
};

#[derive(Debug, Clone)]
pub struct CadenceData {
    pub global_commits_per_week: u32,
    pub author_commits_per_week: Vec<AuthorCommits>,
}

#[derive(Debug)]
pub struct CadencePage {
    pub data: CadenceData,
    pub selected_index: usize,
    pub selected_author: Option<AuthorDetails>,
    pub table_state: TableState,
}

#[derive(Debug, Clone)]

// remove this type
pub struct AuthorCommits {
    pub name: String,
    pub commits_per_week: u32,
}

#[derive(Debug)]
pub struct AuthorDetails {
    pub name: String, // TODO:  fix these props later
    pub commits_per_week: u32,
    pub first_commit: String,
    pub total_commits: u32,
    pub repo_share: f64,
}

impl CadencePage {
    pub fn new(data: CadenceData) -> Self {
        Self {
            data,
            selected_index: 0,
            selected_author: None,
            table_state: TableState::default().with_selected(Some(0)),
        }
    }

    pub fn handle_key(&mut self, key_event: KeyEvent, repo: &KitRepo) {
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.next_index(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_index(),
            KeyCode::Enter => self.select(repo),
            KeyCode::Esc | KeyCode::Backspace => self.unselect(),
            _ => {}
        };
    }

    pub fn next_index(&mut self) {
        if !self.data.author_commits_per_week.is_empty() {
            self.selected_index =
                (self.selected_index + 1) % self.data.author_commits_per_week.len();
            self.table_state.select(Some(self.selected_index));
        }
    }

    pub fn previous_index(&mut self) {
        if !self.data.author_commits_per_week.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.data.author_commits_per_week.len() - 1;
            } else {
                self.selected_index -= 1;
            }

            self.table_state.select(Some(self.selected_index));
        }
    }

    // used to unselect (e.g using Esc)
    pub fn unselect(&mut self) {
        self.selected_author = None;
    }

    // select author from commit list and fetch data for the more_info() window
    pub fn select(&mut self, repo: &KitRepo) {
        if self.selected_author.take().is_some() {
            return;
        }

        let AuthorCommits {
            name,
            commits_per_week,
        } = self.data.author_commits_per_week[self.selected_index].clone();

        let first_commit = CadenceData::author_first_commit(repo, &name)
            .ok()
            .flatten()
            .map(|commit| {
                commit_to_date(&commit)
                    .map(|date| date.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| commit.time().seconds().to_string())
            })
            .unwrap_or_else(String::new);

        let total_commits = repo
            .get_author_commits(&name)
            .map_or(0, |iter| iter.count()) as u32;

        let repo_share = CadenceData::author_repository_share(repo, &name).unwrap_or(0.0);

        let details = AuthorDetails {
            name: name,
            commits_per_week: commits_per_week,
            first_commit,
            total_commits,
            repo_share,
        };
        self.selected_author = Some(details);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default().padding(Padding::horizontal(1));

        frame.render_widget(block.clone(), area);

        let inner_area = block.inner(area);

        let left_constraint = Constraint::Percentage(60);
        let right_constraint = Constraint::Percentage(40);
        let middle_spacer = Constraint::Percentage(2);

        let main_columns = Layout::horizontal([left_constraint, middle_spacer, right_constraint])
            .split(inner_area);

        let left_column = main_columns[0];
        let right_column = main_columns[2];

        self.author_table(frame, left_column);
        self.chart(frame, right_column);

        // show more info frame last, this will draw it on top
        if let Some(details) = &self.selected_author {
            self.more_info(frame, details);
        }
    }

    fn chart(&self, frame: &mut Frame, area: Rect) {
        let mut authors: Vec<(&String, &u32)> = self
            .data
            .author_commits_per_week
            .iter()
            .map(|ac| (&ac.name, &ac.commits_per_week))
            .collect();
        authors.sort_by(|a, b| a.1.cmp(b.1));

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
            .data
            .author_commits_per_week
            .iter()
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
            .block(Block::default().title(" Authors ").borders(Borders::ALL))
            .row_highlight_style(ACCENT)
            .highlight_symbol("> ");

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    pub fn more_info(&self, frame: &mut Frame, details: &AuthorDetails) {
        let area = frame
            .area()
            .centered(Constraint::Percentage(25), Constraint::Percentage(25));

        let title = format!(" {} ", details.name);

        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(ACCENT))
            .title(title)
            .title_style(Color::White)
            .title_alignment(Alignment::Center);

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

        let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Left);

        frame.render_widget(Clear, area); // clear to remove underneath text
        frame.render_widget(paragraph, area);
    }
}

impl CadenceData {
    pub fn author_first_commit<'a>(repo: &'a KitRepo, email: &str) -> Result<Option<Commit<'a>>> {
        let commits = repo.get_author_commits(email)?;
        Ok(commits.last()) // the commit list is in reverse order
    }

    pub fn author_repository_share(repo: &KitRepo, email: &str) -> Result<f64> {
        let author_count = repo.get_author_commits(email)?.count();
        let repo_count = repo.iter_commits()?.count();

        if repo_count == 0 {
            return Ok(0.0);
        }

        let share = (author_count as f64) / (repo_count as f64);

        let percentage = share * 100.0;
        Ok(percentage)
    }

    pub fn author_commits_per_week(repo: &KitRepo, email: &str) -> Result<u32> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .get_author_commits(email)?
            .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
            .collect();

        Ok(commits_per_week(&commit_dates))
    }

    pub fn global_commits_per_week(repo: &KitRepo) -> Result<u32> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .iter_commits()?
            .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
            .collect();

        Ok(commits_per_week(&commit_dates))
    }

    pub fn full_report(repo: &KitRepo) -> Result<Self> {
        let mut cadence = CadenceData {
            global_commits_per_week: Self::global_commits_per_week(repo)?,
            author_commits_per_week: Vec::new(),
        };
        for author in repo.get_authors()? {
            let commit_dates: Vec<DateTime<Utc>> = repo
                .get_author_commits(&author)?
                .filter_map(|commit| DateTime::from_timestamp_secs(commit.time().seconds()))
                .collect();

            cadence.author_commits_per_week.push(AuthorCommits {
                name: author,
                commits_per_week: commits_per_week(&commit_dates),
            });
        }
        cadence
            .author_commits_per_week
            .sort_by(|a, b| b.commits_per_week.cmp(&a.commits_per_week));
        Ok(cadence)
    }
}

fn commits_per_week(commits: &[DateTime<Utc>]) -> u32 {
    match telescope_time(&commits) {
        Some(delta) => {
            let seconds_avg = delta.as_seconds_f32();
            if seconds_avg > 0.0 {
                ((1.0 / seconds_avg) * 60.0 * 60.0 * 24.0 * 7.0) as u32
            } else {
                0.0 as u32
            }
        }
        None => 0.0 as u32,
    }
}

//https://en.wikipedia.org/wiki/Telescoping_series
fn telescope_time(datetimes: &[DateTime<Utc>]) -> Option<TimeDelta> {
    if datetimes.len() < 2 {
        return None;
    }

    // the middle dates all cancel when summing over their differences as pairs
    // and we are left with the first and last only
    let total_duration = *datetimes.first()? - *datetimes.last()?;
    let count = (datetimes.len() - 1) as i32;

    total_duration.checked_div(count)
}

fn commit_to_date(commit: &Commit) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_secs(commit.time().seconds())
}
