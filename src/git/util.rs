use chrono::{DateTime, Utc};
use git2::Commit;

pub fn commit_to_date(commit: &Commit) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_secs(commit.time().seconds())
}
pub fn active_since(commit: &Option<Commit>) -> String {
    if let Some(c) = commit {
        if let Some(commit_date) = commit_to_date(&c) {
            let now = Utc::now();
            let readable_date = commit_date.format("%Y/%m/%d").to_string();

            let duration = now.signed_duration_since(commit_date);
            let years = duration.num_days() as f64 / 365.0;

            format!("active since {} ({:.1} years)", readable_date, years)
        } else {
            "not found".to_string()
        }
    } else {
        "not found".to_string()
    }
}

pub fn last_activity(commit: &Option<Commit>) -> String {
    if let Some(c) = commit {
        if let Some(commit_date) = commit_to_date(&c) {
            let mut hash = c.id().to_string();
            hash.truncate(7);
            let author = c
                .author()
                .email()
                .map_or("no author".to_owned(), |a| a.to_owned());

            let now = Utc::now();

            let duration = now.signed_duration_since(commit_date);
            let hours = duration.num_hours() as f64;

            format!(
                "last activity ({}) by: {} {} hours ago",
                hash, author, hours
            )
        } else {
            "not found".to_string()
        }
    } else {
        "not found".to_string()
    }
}
