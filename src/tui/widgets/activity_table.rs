use std::collections::HashMap;

use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use ratatui::{
    layout::{Constraint, Layout, Offset},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Widget},
};

use crate::git::model::KitCommit;

const DAYS_IN_ORDER: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

pub struct ActivityTable<'c> {
    pub lines: Vec<Line<'c>>,
}
impl<'c> ActivityTable<'c> {
    pub fn new(commits: &'c [KitCommit]) -> Self {
        let mut weekday_commits: HashMap<Weekday, Vec<DateTime<Utc>>> = HashMap::new();
        for commit in commits {
            if let Some(date) = commit.date {
                let weekday = date.weekday();

                weekday_commits.entry(weekday).or_default().push(date);
            }
        }

        let lines: Vec<Line<'c>> = DAYS_IN_ORDER
            .iter()
            .map(|day| {
                let day_commits = weekday_commits.get(day).map(Vec::as_slice).unwrap_or(&[]);
                Self::day_bar(day, day_commits)
            })
            .collect();

        Self { lines }
    }

    fn day_bar(_day: &Weekday, day_commits: &[DateTime<Utc>]) -> Line<'c> {
        let mut commits_per_hour = [0u32; 48];
        for commit in day_commits {
            let hour = commit.hour() as usize;
            commits_per_hour[hour] += 1;
        }
        let peak_value = *commits_per_hour.iter().max().unwrap_or(&0);
        let mut spans: Vec<Span> = vec![];

        for &cell_value in commits_per_hour.iter() {
            let cell = BarCell::new(cell_value, peak_value);
            spans.push(cell.to_span());
        }
        Line::from(spans)
    }
}

impl<'c> Widget for ActivityTable<'c> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let vertical_limit = Layout::vertical([Constraint::Length(9)]).split(area);

        let table_layout = Layout::horizontal([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(vertical_limit[0]);
        let block = Block::bordered();

        let inside_block = block.inner(table_layout[2]);

        block.render(table_layout[2], buf);

        let mut i: usize = 0;
        for line in self.lines {
            line.render(inside_block.offset(Offset { x: 0, y: i as i32 }), buf);

            Span::raw(format!("{} \n", DAYS_IN_ORDER[i].to_string())).render(
                table_layout[0].offset(Offset {
                    x: 0,
                    y: (i + 1) as i32, // this is outside the border. border has 1px diff so we offset for that too
                }),
                buf,
            );
            i += 1;
        }

        // // write time axis
        Line::from(vec![
            format!("00  02  04  06  08  10  12  14  16  18  20  22  ").into(),
        ])
        .render(
            table_layout[2].offset(Offset {
                x: 0,
                y: (i + 2) as i32,
            }),
            buf,
        );
    }
}

struct BarCell {
    style: Style,
    char: &'static str,
}

impl Default for BarCell {
    fn default() -> Self {
        Self {
            style: Style::default().fg(Color::Rgb(40, 44, 52)),
            char: "░░",
        }
    }
}

impl BarCell {
    pub fn new(cell_value: u32, peak_value: u32) -> Self {
        if cell_value == 0 {
            return BarCell::default();
        }

        let ratio = cell_value as f32 / peak_value as f32;

        if ratio > 0.66 {
            BarCell {
                style: Style::default().fg(Color::Rgb(220, 138, 120)),
                char: "██",
            }
        } else if ratio > 0.33 {
            BarCell {
                style: Style::default().fg(Color::Rgb(166, 105, 90)),
                char: "██",
            }
        } else {
            BarCell {
                style: Style::default().fg(Color::Rgb(92, 80, 77)),
                char: "██",
            }
        }
    }

    pub fn to_span(&self) -> Span<'static> {
        Span::styled(self.char, self.style)
    }
}
