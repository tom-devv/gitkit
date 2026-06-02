use std::{collections::HashSet, path::Path};

use git2::{Commit, Diff, Error, Oid, Repository, StatusOptions, Statuses};

use crate::git::status::KitStatus;

pub struct KitRepo {
    pub inner: Repository,
}

impl KitRepo {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<KitRepo, Error> {
        let repo = Repository::open(path)?;
        Ok(KitRepo { inner: repo })
    }

    // use iter_commits unless Vec<Commit> is needed
    pub fn get_all_commits<'a>(&'a self) -> Result<Vec<Commit<'a>>, Error> {
        let revwalk: Vec<Oid> = match self.inner.revwalk() {
            Ok(mut walk) => {
                if let Err(e) = walk.push_head() {
                    panic!("Failed to push HEAD to revwalk: {}", e);
                }

                walk.into_iter().flatten().collect()
            }
            Err(e) => panic!("Failed to create revwalk: {}", e),
        };

        let commits: Vec<Commit<'a>> = revwalk
            .iter()
            .map(|oid| self.inner.find_commit(*oid).unwrap())
            .collect();

        Ok(commits)
    }

    pub fn iter_commits(&self) -> Result<impl Iterator<Item = Commit<'_>>, git2::Error> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.push_head()?;

        let repo_ref = &self.inner;

        Ok(revwalk
            .flatten()
            .filter_map(move |oid| repo_ref.find_commit(oid).ok()))
    }

    pub fn get_authors(&self) -> Result<HashSet<String>, git2::Error> {
        let mut authors: HashSet<String> = HashSet::new();
        for commit in self.iter_commits()? {
            let commiter = commit.author();
            if let Ok(email) = commiter.email() {
                authors.insert(email.to_owned());
            }
        }

        Ok(authors)
    }

    pub fn get_author_commits(
        &self,
        email: &str,
    ) -> Result<impl Iterator<Item = Commit<'_>>, git2::Error> {
        let email_owned = email.to_string();
        let iter = self
            .iter_commits()?
            .filter(move |commit| commit.author().email().map_or(false, |e| e == email_owned));

        Ok(iter)
    }

    pub fn get_diff(
        &self,
        parent: Option<&Commit>,
        current: Option<&Commit>,
    ) -> Result<Diff<'_>, git2::Error> {
        let parent_tree = parent.map(|c| c.tree()).transpose()?;
        let current_tree = current.map(|c| c.tree()).transpose()?;

        let diff =
            self.inner
                .diff_tree_to_tree(parent_tree.as_ref(), current_tree.as_ref(), None)?;
        Ok(diff)
    }

    pub fn get_parent_diff(&self, commit: &Commit) -> Result<Diff<'_>, git2::Error> {
        let parent_commit = match commit.parent(0) {
            Ok(parent) => Some(parent),
            Err(_) => None,
        };

        self.get_diff(parent_commit.as_ref(), Some(commit))
    }

    pub fn iter_all_diffs(&self) -> Result<impl Iterator<Item = Diff<'_>>, git2::Error> {
        let diffs = self
            .iter_commits()?
            .filter_map(|commit| self.get_parent_diff(&commit).ok());
        Ok(diffs)
    }

    pub fn current_branch(&self) -> Result<String, git2::Error> {
        let head = self.inner.head()?;
        head.shorthand().map(|s| s.to_owned())
    }

    pub fn get_status(&self) -> KitStatus {
        KitStatus::new(&self)
    }

    fn get_raw_status(&self) -> Result<Statuses<'_>, git2::Error> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = self.inner.statuses(Some(&mut opts))?;

        Ok(statuses)
    }

    pub fn is_dirty(&self) -> Result<bool, git2::Error> {
        let statuses = self.get_raw_status()?;
        Ok(!statuses.is_empty())
    }
}
