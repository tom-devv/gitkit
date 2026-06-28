use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::git::kit::KitRepo;
use crate::git::model::KitCommit;

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
    pub all_commits: Vec<KitCommit>,
}

impl CadenceData {
    pub fn new(repo: &KitRepo) -> Self {
        let mut author_map: HashMap<String, Vec<KitCommit>> = HashMap::new();
        let mut total_repo_commits = 0;

        let mut global_earliest: Option<DateTime<Utc>> = None;
        let mut global_latest: Option<DateTime<Utc>> = None;

        if let Ok(commits) = repo.iter_commits() {
            for commit in commits {
                total_repo_commits += 1;

                if let Some(date) = commit.date {
                    global_earliest = Some(global_earliest.map_or(date, |e| e.min(date)));
                    global_latest = Some(global_latest.map_or(date, |l| l.max(date)));
                }

                let author_key = commit.email.clone();

                author_map
                    .entry(author_key)
                    .or_insert_with(Vec::new)
                    .push(commit);
            }
        }

        let lifespan_weeks = Self::calculate_lifespan_weeks(global_earliest, global_latest);

        let global_commits_per_week = if lifespan_weeks > 0.0 {
            (total_repo_commits as f32) / lifespan_weeks
        } else {
            0.0
        };

        let mut author_details = Vec::with_capacity(author_map.len());

        for (author_name, commits) in author_map {
            let author_total = commits.len() as u32;

            let repo_share = if total_repo_commits > 0 {
                (author_total as f64 / total_repo_commits as f64) * 100.0
            } else {
                0.0
            };

            let first_commit = commits.last().and_then(|c| c.date).unwrap_or_default();

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
                all_commits: commits,
            });
        }

        author_details.sort_by(|a, b| b.commits_per_week.partial_cmp(&a.commits_per_week).unwrap());

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
