use chrono::Weekday;
use ratatui::{
    layout::{Constraint, Layout, Offset},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Widget},
};

use crate::git::metrics::cadence::Activity;

const DAYS_IN_ORDER: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

pub struct ActivityTable {
    pub lines: Vec<Line<'static>>,
}
impl ActivityTable {
    pub fn new(activity: &Activity) -> Self {
        let lines = activity.iter().map(Self::day_bar).collect();

        Self { lines }
    }

    fn day_bar(commits_per_hour: &[u32; 24]) -> Line<'static> {
        let peak_value = *commits_per_hour.iter().max().unwrap_or(&0);
        let spans: Vec<Span> = commits_per_hour
            .iter()
            .map(|&cell_value| BarCell::new(cell_value, peak_value).to_span())
            .collect();
        Line::from(spans)
    }
}

impl Widget for ActivityTable {
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
