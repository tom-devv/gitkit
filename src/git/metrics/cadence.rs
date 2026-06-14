use chrono::{DateTime, TimeDelta, Utc};

use crate::error::Result;
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
}

impl CadenceData {
    // TODO refactor this into smaller methods
    pub fn new(repo: &KitRepo) -> Self {
        let mut cadence = CadenceData {
            global_commits_per_week: Self::global_commits_per_week(repo).unwrap_or(0.0),
            author_details: Vec::new(),
        };
        for author in repo.get_authors().unwrap_or_default() {
            if let Ok(author_commits) = repo.get_author_commits(&author) {
                let mut total_commits = 0;
                let commit_dates: Vec<DateTime<Utc>> = author_commits
                    .filter_map(|commit| {
                        total_commits += 1; // avoids consuming the iter twice
                        commit.date
                    })
                    .collect();

                let first_commit: DateTime<Utc> = Self::author_first_commit(repo, &author)
                    .ok()
                    .flatten()
                    .map(|z| z.date)
                    .flatten()
                    .unwrap_or(DateTime::default());

                let repo_share = Self::author_repository_share(repo, &author).unwrap_or(0.0);

                cadence.author_details.push(AuthorDetails {
                    name: author.clone(),
                    commits_per_week: commits_per_week(&commit_dates, repo),
                    first_commit,
                    total_commits,
                    repo_share,
                });
            }
        }
        cadence
            .author_details
            .sort_by(|a, b| b.commits_per_week.partial_cmp(&a.commits_per_week).unwrap());
        cadence
    }

    pub fn author_first_commit<'a>(repo: &'a KitRepo, email: &str) -> Result<Option<KitCommit>> {
        let commits = repo.get_author_commits(email)?;
        Ok(commits.last()) // reverse order
    }

    pub fn author_last_commit<'a>(repo: &'a KitRepo, email: &str) -> Result<Option<KitCommit>> {
        let mut commits = repo.get_author_commits(email)?;
        Ok(commits.next())
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

    pub fn author_commits_per_week(repo: &KitRepo, email: &str) -> Result<f32> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .get_author_commits(email)?
            .filter_map(|commit| commit.date)
            .collect();

        Ok(commits_per_week(&commit_dates, repo))
    }

    pub fn global_commits_per_week(repo: &KitRepo) -> Result<f32> {
        let commit_dates: Vec<DateTime<Utc>> = repo
            .iter_commits()?
            .filter_map(|commit| commit.date)
            .collect();

        Ok(commits_per_week(&commit_dates, repo))
    }
}

const WEEK_SOMETHING: f32 = 60.0 * 60.0 * 24.0 * 7.0;

fn commits_per_week(commits: &[DateTime<Utc>], repo: &KitRepo) -> f32 {
    let first_commit = repo.get_first_commit();
    let last_commit = repo.get_last_commit();

    if let (Ok(first), Ok(last)) = (first_commit, last_commit) {
        if let (Some(start), Some(end)) = (first.date, last.date) {
            let lifespan_seconds = (start - end).num_seconds() as f32;
            let lifespan_weeks = (lifespan_seconds / (60.0 * 60.0 * 24.0 * 7.0)).max(1.0);

            (commits.len() as f32 / lifespan_weeks) as f32
        } else {
            0.0
        }
    } else {
        0.0
    }
}

fn commits_per_something(commits: &[DateTime<Utc>], something: f64, repo: &KitRepo) -> f64 {
    let first_commit = repo.get_first_commit();
    let last_commit = repo.get_last_commit();

    if let (Ok(first), Ok(last)) = (first_commit, last_commit) {
        if let (Some(start), Some(end)) = (first.date, last.date) {
            let lifespan_seconds = (start - end).num_seconds() as f64;
            let lifespan_weeks = (lifespan_seconds / something).max(1.0);

            (commits.len() as f64 / lifespan_weeks) as f64
        } else {
            0.0
        }
    } else {
        0.0
    }
}

//https://en.wikipedia.org/wiki/Telescoping_series
// this is only useful to determine the avg time between commits
// and will be heavily skewed for users with few commits
fn _telescope_time(datetimes: &[DateTime<Utc>]) -> Option<TimeDelta> {
    if datetimes.len() < 2 {
        return None;
    }

    // the middle dates all cancel when summing over their differences as pairs
    // and we are left with the first and last only
    let total_duration = *datetimes.first()? - *datetimes.last()?;
    let count = (datetimes.len() - 1) as i32;

    total_duration.checked_div(count)
}
