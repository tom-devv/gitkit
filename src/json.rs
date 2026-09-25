use std::io::{self, Write};

use chrono::{DateTime, SecondsFormat, Utc};
use clap::ValueEnum;
use serde::Serialize;

use crate::{
    error::Result,
    git::{
        kit::KitRepo,
        metrics::{
            cadence::{Activity, CadenceData},
            home::HomeData,
            silo::SiloData,
        },
        model::KitCommit,
    },
};

// the pages that can be exported, mirrors the tui tabs
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Section {
    Home,
    Cadence,
    Silo,
}

// output schema, kept separate from the metric structs so
// internal changes don't silently change the json shape
#[derive(Serialize)]
struct Report {
    gitkit_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    home: Option<HomeJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cadence: Option<CadenceJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    silo: Option<SiloJson>,
}

#[derive(Serialize)]
struct HomeJson {
    repo_name: String,
    current_branch: String,
    total_commits: u32,
    first_commit: Option<CommitJson>,
    last_commit: Option<CommitJson>,
    status: Vec<StatusJson>,
}

#[derive(Serialize)]
struct CommitJson {
    id: String,
    author_email: String,
    date: Option<String>,
    timestamp: i64,
}

#[derive(Serialize)]
struct StatusJson {
    path: String,
    // None when the file is unchanged on that side
    index: Option<&'static str>,
    worktree: Option<&'static str>,
}

#[derive(Serialize)]
struct CadenceJson {
    global_commits_per_week: f32,
    authors: Vec<AuthorJson>,
}

#[derive(Serialize)]
struct AuthorJson {
    email: String,
    total_commits: u32,
    commits_per_week: f32,
    repo_share_percent: f64,
    first_commit: String,
    activity: ActivityJson,
}

// commit counts per hour of day (utc, index 0 = 00:00)
#[derive(Serialize)]
struct ActivityJson {
    mon: [u32; 24],
    tue: [u32; 24],
    wed: [u32; 24],
    thu: [u32; 24],
    fri: [u32; 24],
    sat: [u32; 24],
    sun: [u32; 24],
}

#[derive(Serialize)]
struct SiloJson {
    files: Vec<FileSiloJson>,
}

#[derive(Serialize)]
struct FileSiloJson {
    path: String,
    gatekeeper: String,
    contributors: u16,
    risk_percent: u8,
    total_churn: usize,
    // sorted by churn, highest first
    authors: Vec<AuthorChurnJson>,
}

#[derive(Serialize)]
struct AuthorChurnJson {
    email: String,
    churn: usize,
}

pub fn print_report(repo: &KitRepo, sections: &[Section]) -> Result<()> {
    let wants = |section| sections.is_empty() || sections.contains(&section);

    let report = Report {
        gitkit_version: env!("CARGO_PKG_VERSION"),
        home: wants(Section::Home).then(|| HomeJson::from(HomeData::new(repo))),
        cadence: wants(Section::Cadence).then(|| CadenceJson::from(CadenceData::new(repo))),
        silo: wants(Section::Silo).then(|| SiloJson::from(SiloData::new(repo))),
    };

    let mut stdout = io::stdout().lock();
    let written = serde_json::to_writer_pretty(&mut stdout, &report)
        .map_err(io::Error::from)
        .and_then(|_| writeln!(stdout));

    match written {
        // the reader went away (e.g. `| head`), nothing left to do
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => Ok(other?),
    }
}

fn format_date(date: DateTime<Utc>) -> String {
    date.to_rfc3339_opts(SecondsFormat::Secs, true)
}

impl From<KitCommit> for CommitJson {
    fn from(commit: KitCommit) -> Self {
        Self {
            id: commit.id,
            author_email: commit.email,
            date: commit.date.map(format_date),
            timestamp: commit.time_seconds,
        }
    }
}

impl From<HomeData> for HomeJson {
    fn from(data: HomeData) -> Self {
        let status = data
            .status
            .files
            .into_iter()
            .map(|(path, status)| StatusJson {
                path,
                index: index_state(status),
                worktree: worktree_state(status),
            })
            .collect();

        Self {
            repo_name: data.repo_name,
            current_branch: data.current_branch,
            total_commits: data.total_commits,
            first_commit: data.first_commit.map(CommitJson::from),
            last_commit: data.last_commit.map(CommitJson::from),
            status,
        }
    }
}

fn index_state(status: git2::Status) -> Option<&'static str> {
    use git2::Status as S;
    match status {
        s if s.contains(S::INDEX_NEW) => Some("new"),
        s if s.contains(S::INDEX_MODIFIED) => Some("modified"),
        s if s.contains(S::INDEX_DELETED) => Some("deleted"),
        s if s.contains(S::INDEX_RENAMED) => Some("renamed"),
        s if s.contains(S::INDEX_TYPECHANGE) => Some("typechange"),
        _ => None,
    }
}

fn worktree_state(status: git2::Status) -> Option<&'static str> {
    use git2::Status as S;
    match status {
        s if s.contains(S::WT_NEW) => Some("untracked"),
        s if s.contains(S::WT_MODIFIED) => Some("modified"),
        s if s.contains(S::WT_DELETED) => Some("deleted"),
        s if s.contains(S::WT_RENAMED) => Some("renamed"),
        s if s.contains(S::WT_TYPECHANGE) => Some("typechange"),
        s if s.contains(S::CONFLICTED) => Some("conflicted"),
        _ => None,
    }
}

impl From<CadenceData> for CadenceJson {
    fn from(data: CadenceData) -> Self {
        let authors = data
            .author_details
            .into_iter()
            .map(|author| AuthorJson {
                email: author.name,
                total_commits: author.total_commits,
                commits_per_week: author.commits_per_week,
                repo_share_percent: author.repo_share,
                first_commit: format_date(author.first_commit),
                activity: ActivityJson::from(author.activity),
            })
            .collect();

        Self {
            global_commits_per_week: data.global_commits_per_week,
            authors,
        }
    }
}

impl From<Activity> for ActivityJson {
    fn from([mon, tue, wed, thu, fri, sat, sun]: Activity) -> Self {
        Self {
            mon,
            tue,
            wed,
            thu,
            fri,
            sat,
            sun,
        }
    }
}

impl From<SiloData> for SiloJson {
    fn from(data: SiloData) -> Self {
        let files = data
            .files
            .into_iter()
            .map(|file| {
                let mut authors: Vec<AuthorChurnJson> = file
                    .author_churn
                    .into_iter()
                    .map(|(email, churn)| AuthorChurnJson { email, churn })
                    .collect();
                authors.sort_by(|a, b| b.churn.cmp(&a.churn).then_with(|| a.email.cmp(&b.email)));

                FileSiloJson {
                    path: file.file,
                    gatekeeper: file.gatekeeper,
                    contributors: file.contributors,
                    risk_percent: file.risk,
                    total_churn: file.total_churn,
                    authors,
                }
            })
            .collect();

        Self { files }
    }
}
