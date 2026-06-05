
use chrono::{DateTime, Utc};
use git2::Commit;
use ratatui::{
    style::{Stylize, palette::material::WHITE},
    text::Line,
};

use crate::tui::{ACCENT, ACCENT_TEXT};

const NOT_FOUND: &str = "not found";

pub fn not_found() -> Line<'static> {
    Line::from(NOT_FOUND.to_owned().fg(ACCENT_TEXT))
}

pub fn commit_to_date(commit: &Commit) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_secs(commit.time().seconds())
}

pub fn active_since(commit: &Option<Commit>) -> Line<'static> {
    if let Some(c) = commit {
        if let Some(commit_date) = commit_to_date(c) {
            let now = Utc::now();
            let readable_date = commit_date.format("%Y/%m/%d").to_string();

            let duration = now.signed_duration_since(commit_date);
            let years = duration.num_days() as f64 / 365.0;

            Line::from(vec![
                "active since: ".fg(ACCENT).bold(),
                format!("{} ({:.1} years)", readable_date, years).fg(WHITE),
            ])
        } else {
            not_found()
        }
    } else {
        not_found()
    }
}

pub fn last_activity(commit: &Option<Commit>) -> Line<'static> {
    if let Some(c) = commit {
        if let Some(commit_date) = commit_to_date(c) {
            let mut hash = c.id().to_string();
            hash.truncate(7);
            let author = c
                .author()
                .email()
                .map_or("no author".to_owned(), |a| a.to_owned());

            let now = Utc::now();

            let duration = now.signed_duration_since(commit_date);
            let hours = duration.num_hours() as f64;

            Line::from(vec![
                "last activity: ".fg(ACCENT).bold(),
                format!("({}) ", hash).fg(ACCENT_TEXT),
                format!("{} ", hours).fg(WHITE),
                "hours ago ".fg(ACCENT_TEXT),
                "by: ".fg(ACCENT_TEXT),
                format!("{} ", author).fg(WHITE),
            ])
        } else {
            not_found()
        }
    } else {
        not_found()
    }
}
