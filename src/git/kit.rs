use std::{collections::HashSet, path::Path};

use git2::{Commit, Diff, DiffOptions, Error, Oid, Repository, StatusOptions, Statuses};

use crate::git::{self, contributions::Contribution, model::KitCommit, status::KitStatus};

pub struct KitRepo {
    pub inner: Repository,
}

impl<'repo> KitRepo {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<KitRepo, Error> {
        let repo = Repository::open(path)?;
        Ok(KitRepo { inner: repo })
    }

    pub fn change_branch(&self, branch_name: &str) -> Result<(), git2::Error> {
        let branch = self
            .inner
            .find_branch(branch_name, git2::BranchType::Local)?;
        let reference = branch.get();

        let obj = reference.peel_to_commit()?;
        self.inner.set_head(reference.name().unwrap())?;

        self.inner.checkout_tree(obj.as_object(), None)?;

        Ok(())
    }

    pub fn list_branch(&self) -> Result<(), git2::Error> {
        let branches = self.inner.branches(Some(git2::BranchType::Local))?;
        branches.filter_map(|x| x.ok()).for_each(|y| {
            let name =
                y.0.name()
                    .unwrap_or(Some("No name found"))
                    .unwrap_or_default();

            println!("{}", name);
        });
        Ok(())
    }

    // use iter_commits unless Vec<Commit> is needed
    pub fn get_all_commits<'a>(&'a self) -> Result<Vec<KitCommit>, Error> {
        let revwalk: Vec<Oid> = match self.inner.revwalk() {
            Ok(mut walk) => {
                if let Err(e) = walk.push_head() {
                    panic!("Failed to push HEAD to revwalk: {}", e);
                }

                walk.into_iter().flatten().collect()
            }
            Err(e) => panic!("Failed to create revwalk: {}", e),
        };

        let commits: Vec<KitCommit> = revwalk
            .iter()
            .map(|oid| self.inner.find_commit(*oid).unwrap())
            .map(|c| KitCommit::from_git2(&c))
            .collect();

        Ok(commits)
    }

    pub fn iter_commits(&self) -> Result<impl Iterator<Item = KitCommit>, git2::Error> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.push_head()?;

        let repo_ref = &self.inner;

        Ok(revwalk
            .flatten()
            .filter_map(move |oid| repo_ref.find_commit(oid).ok())
            .map(|c| KitCommit::from_git2(&c)))
    }

    pub fn get_authors(&self) -> Result<HashSet<String>, git2::Error> {
        let mut authors: HashSet<String> = HashSet::new();
        for commit in self.iter_commits()? {
            authors.insert(commit.email);
        }

        Ok(authors)
    }

    pub fn get_author_commits(
        &self,
        email: &str,
    ) -> Result<impl Iterator<Item = KitCommit>, git2::Error> {
        let email_owned = email.to_string();
        let iter = self
            .iter_commits()?
            .filter(move |commit| commit.email == email_owned);

        Ok(iter)
    }

    pub fn get_diff(
        &self,
        parent: Option<&Commit>,
        current: Option<&Commit>,
        opts: Option<&mut DiffOptions>,
    ) -> Result<Diff<'_>, git2::Error> {
        let parent_tree = parent.map(|c| c.tree()).transpose()?;
        let current_tree = current.map(|c| c.tree()).transpose()?;

        let diff =
            self.inner
                .diff_tree_to_tree(parent_tree.as_ref(), current_tree.as_ref(), opts)?;
        Ok(diff)
    }

    pub fn get_parent_diff(
        &self,
        commit: &Commit<'repo>,
        opts: Option<&mut DiffOptions>,
    ) -> Result<Diff<'_>, git2::Error> {
        let parent_commit = match commit.parent(0) {
            Ok(parent) => Some(parent),
            Err(_) => None,
        };

        self.get_diff(parent_commit.as_ref(), Some(commit), opts)
    }

    // get all diffs exlcuding merge commits
    pub fn iter_diff_history<'a>(
        &'a self,
    ) -> Result<impl Iterator<Item = (KitCommit, Diff<'a>)> + 'a, git2::Error> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.push_head()?;

        let repo = &self.inner;

        let diff_iter = revwalk.filter_map(move |oid_result| {
            let oid = oid_result.ok()?;
            let commit = repo.find_commit(oid).ok()?;

            // Skip merge commits
            if commit.parent_count() > 1 {
                return None;
            }

            let commit_tree = commit.tree().ok()?;

            let parent_tree = if commit.parent_count() == 1 {
                commit.parent(0).ok()?.tree().ok()
            } else {
                None
            };

            let mut diff_options = DiffOptions::new();
            diff_options.ignore_whitespace(true);

            let diff = repo
                .diff_tree_to_tree(
                    parent_tree.as_ref(),
                    Some(&commit_tree),
                    Some(&mut diff_options),
                )
                .ok()?;

            Some((KitCommit::from_git2(&commit), diff))
        });

        Ok(diff_iter)
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
