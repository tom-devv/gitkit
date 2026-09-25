use std::collections::HashMap;

use chrono::{DateTime, Datelike, Timelike, Utc};

use crate::git::kit::KitRepo;

// commit counts indexed by [weekday from monday][hour of day]
pub type Activity = [[u32; 24]; 7];

#[derive(Debug, Clone)]
pub struct CadenceData {
    pub global_commits_per_week: f32,
    pub author_details: Vec<AuthorDetails>,
}

#[derive(Debug, Clone)]
pub struct AuthorDetails {
    pub name: String,
    pub commits_per_week: f32,
    pub first_commit: DateTime<Utc>,
    pub total_commits: u32,
    pub repo_share: f64,
    pub activity: Activity,
}

// running totals per author while walking history
struct AuthorAcc {
    total: u32,
    // commit iter is reversed, so the last one seen is the earliest
    earliest: Option<DateTime<Utc>>,
    activity: Activity,
}

impl CadenceData {
    pub fn new(repo: &KitRepo) -> Self {
        let mut author_map: HashMap<String, AuthorAcc> = HashMap::new();
        let mut total_repo_commits = 0;

        let mut global_earliest: Option<DateTime<Utc>> = None;
        let mut global_latest: Option<DateTime<Utc>> = None;

        if let Ok(commits) = repo.iter_raw_commits() {
            for commit in commits {
                total_repo_commits += 1;

                let date = DateTime::from_timestamp_secs(commit.time().seconds());
                if let Some(date) = date {
                    global_earliest = Some(global_earliest.map_or(date, |e| e.min(date)));
                    global_latest = Some(global_latest.map_or(date, |l| l.max(date)));
                }

                let author = commit.author();
                let email = author.email().unwrap_or("Unknown");
                // only allocate the key the first time we see an author
                let acc = match author_map.get_mut(email) {
                    Some(acc) => acc,
                    None => author_map.entry(email.to_string()).or_insert(AuthorAcc {
                        total: 0,
                        earliest: None,
                        activity: [[0; 24]; 7],
                    }),
                };

                acc.total += 1;
                acc.earliest = date;
                if let Some(date) = date {
                    acc.activity[date.weekday().num_days_from_monday() as usize]
                        [date.hour() as usize] += 1;
                }
            }
        }

        let lifespan_weeks = Self::calculate_lifespan_weeks(global_earliest, global_latest);

        let global_commits_per_week = if lifespan_weeks > 0.0 {
            (total_repo_commits as f32) / lifespan_weeks
        } else {
            0.0
        };

        let mut author_details = Vec::with_capacity(author_map.len());

        for (author_name, acc) in author_map {
            let author_total = acc.total;

            let repo_share = if total_repo_commits > 0 {
                (author_total as f64 / total_repo_commits as f64) * 100.0
            } else {
                0.0
            };

            let first_commit = acc.earliest.unwrap_or_default();

            let commits_per_week = if lifespan_weeks > 0.0 {
                (author_total as f32) / lifespan_weeks
            } else {
                0.0
            };

            author_details.push(AuthorDetails {
                name: author_name,
                commits_per_week,
                first_commit,
                total_commits: author_total,
                repo_share,
                activity: acc.activity,
            });
        }

        author_details.sort_by(|a, b| {
            b.commits_per_week
                .total_cmp(&a.commits_per_week)
                .then_with(|| a.name.cmp(&b.name))
        });

        CadenceData {
            global_commits_per_week,
            author_details,
        }
    }

    fn calculate_lifespan_weeks(
        earliest: Option<DateTime<Utc>>,
        latest: Option<DateTime<Utc>>,
    ) -> f32 {
        if let (Some(start), Some(end)) = (earliest, latest) {
            let lifespan_seconds = (end - start).num_seconds().abs() as f32;
            (lifespan_seconds / (60.0 * 60.0 * 24.0 * 7.0)).max(1.0)
        } else {
            1.0
        }
    }
}
