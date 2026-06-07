use chrono::{DateTime, Utc};
use git2::Commit;

pub struct KitCommit {
    pub id: String,
    pub email: String,
    pub date: Option<DateTime<Utc>>,
    pub time_seconds: i64,
}

impl KitCommit {
    pub fn from_git2(commit: &Commit) -> Self {
        let time_seconds = commit.time().seconds();
        let date = Self::to_date(commit);
        Self {
            id: commit.id().to_string(),
            email: commit.author().email().unwrap_or("Unknown").to_string(),
            date,
            time_seconds,
        }
    }

    fn to_date(commit: &Commit) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_secs(commit.time().seconds())
    }
}
