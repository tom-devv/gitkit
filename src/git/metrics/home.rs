use git2::Oid;

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

        let (total_commits, first_commit, last_commit) = Self::walk_history(repo);

        let status = repo.get_status();

        HomeData {
            repo_name,
            current_branch,
            total_commits,
            status,
            first_commit,
            last_commit,
        }
    }

    // single pass over the history: count oids and only build
    // KitCommits for the two ends (commit iter is reversed)
    fn walk_history(repo: &KitRepo) -> (u32, Option<KitCommit>, Option<KitCommit>) {
        let Ok(mut revwalk) = repo.inner.revwalk() else {
            return (0, None, None);
        };
        if revwalk.push_head().is_err() {
            return (0, None, None);
        }

        let mut total: usize = 0;
        let mut newest: Option<Oid> = None;
        let mut oldest: Option<Oid> = None;
        for oid in revwalk.flatten() {
            newest.get_or_insert(oid);
            oldest = Some(oid);
            total += 1;
        }

        let to_commit = |oid: Option<Oid>| {
            oid.and_then(|oid| repo.inner.find_commit(oid).ok())
                .map(|c| KitCommit::from_git2(&c))
        };

        (
            total.try_into().unwrap_or(u32::MAX),
            to_commit(oldest),
            to_commit(newest),
        )
    }
}
