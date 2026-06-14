use crate::git::{kit::KitRepo, model::KitCommit, status::KitStatus};

pub struct HomeData {
    pub repo_name: String, // directory name
    pub current_branch: String,
    pub total_commits: u32,
    pub status: KitStatus,
    pub first_commit: Option<KitCommit>,
    pub last_commit: Option<KitCommit>,
}

impl HomeData {
    pub fn new(repo: &KitRepo) -> Self {
        let workdir = repo.inner.workdir().unwrap_or_else(|| repo.inner.path());

        let repo_name = workdir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unkown workdir".to_string());
        let current_branch = repo
            .current_branch()
            .unwrap_or_else(|_| "not found".to_owned());

        let total_commits: u32 = repo
            .iter_commits()
            .map_or(0, |iter| iter.count())
            .try_into()
            .unwrap_or(u32::MAX);

        let status = repo.get_status();

        // commit iter is reversed
        let first_commit: Option<KitCommit> =
            repo.iter_commits().map_or(None, |commits| commits.last());

        let last_commit = repo
            .iter_commits()
            .map_or(None, |mut commits| commits.next());

        HomeData {
            repo_name,
            current_branch,
            total_commits,
            status,
            first_commit,
            last_commit,
        }
    }
}
