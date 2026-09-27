use std::collections::HashMap;

use chrono::{DateTime, Utc};
use git2::{BranchType, Oid, ReferenceType, Repository, Tree};

use crate::error::Result;
use crate::git::{kit::KitRepo, model::KitCommit};

// branches without a commit for this long are flagged as stale
pub const STALE_AFTER_DAYS: i64 = 90;

// fallbacks when the remote has no default branch (origin/HEAD) set
const BASE_CANDIDATES: [&str; 2] = ["main", "master"];

// how many base commits past the fork point are checked for a squash merge
const MAX_SQUASH_SCAN: usize = 2000;

#[derive(Default)]
pub struct BranchData {
    // the branch everything is compared against, e.g. "origin/main"
    pub base: Option<String>,
    pub branches: Vec<BranchInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchKind {
    Local,
    Remote,
}

// ordered by how much a branch needs cleaning up, most first
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BranchStatus {
    // everything on it is already in base, safe to delete
    Merged,
    // tracked a remote branch that has since been deleted
    Gone,
    // unmerged work nobody has touched in a while
    Stale,
    Active,
}

impl BranchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BranchStatus::Merged => "merged",
            BranchStatus::Gone => "gone",
            BranchStatus::Stale => "stale",
            BranchStatus::Active => "active",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub kind: BranchKind,
    pub is_head: bool,
    pub last_commit: KitCommit,
    // commits on the branch that are not in base, and vice versa
    pub ahead: usize,
    pub behind: usize,
    // in base, either as ancestor or squashed into a single commit
    pub merged: bool,
    // merged via squash, so ahead is still > 0
    pub squash_merged: bool,
    pub stale: bool,
    // local branches only, e.g. "origin/feat/x"
    pub upstream: Option<String>,
    pub upstream_gone: bool,
}

impl BranchInfo {
    pub fn status(&self) -> BranchStatus {
        if self.merged {
            BranchStatus::Merged
        } else if self.upstream_gone {
            BranchStatus::Gone
        } else if self.stale {
            BranchStatus::Stale
        } else {
            BranchStatus::Active
        }
    }
}

impl BranchData {
    pub fn new(repo: &KitRepo) -> Self {
        Self::at(repo, Utc::now()).unwrap_or_default()
    }

    // `now` decides staleness, passed in so results are reproducible
    pub fn at(repo: &KitRepo, now: DateTime<Utc>) -> Result<Self> {
        let base = Self::find_base(repo);
        let base_oid = base.as_ref().map(|(_, oid)| *oid);
        let base_ref = base.as_ref().map(|(name, _)| name.as_str());

        let stale_cutoff = now.timestamp() - STALE_AFTER_DAYS * 24 * 60 * 60;

        let mut patch_ids = PatchIds::default();
        let mut branches = Vec::new();
        for (branch, branch_type) in repo.inner.branches(None)?.flatten() {
            let reference = branch.get();
            // skip origin/HEAD and friends, they just point at another branch
            if reference.kind() != Some(ReferenceType::Direct) {
                continue;
            }
            let Ok(ref_name) = reference.name() else {
                continue;
            };
            if Some(ref_name) == base_ref {
                continue;
            }
            let Ok(commit) = reference.peel_to_commit() else {
                continue;
            };

            let (ahead, behind) = match base_oid {
                Some(base_oid) => repo
                    .inner
                    .graph_ahead_behind(commit.id(), base_oid)
                    .unwrap_or_default(),
                None => (0, 0),
            };

            let squash_merged = match base_oid {
                Some(base_oid) if ahead > 0 => {
                    is_squash_merged(&repo.inner, commit.id(), base_oid, &mut patch_ids)
                }
                _ => false,
            };

            let (upstream, upstream_gone) = match branch_type {
                BranchType::Local => Self::upstream(repo, ref_name),
                BranchType::Remote => (None, false),
            };

            branches.push(BranchInfo {
                name: reference.shorthand().unwrap_or(ref_name).to_string(),
                kind: match branch_type {
                    BranchType::Local => BranchKind::Local,
                    BranchType::Remote => BranchKind::Remote,
                },
                is_head: branch.is_head(),
                last_commit: KitCommit::from_git2(&commit),
                ahead,
                behind,
                merged: (base_oid.is_some() && ahead == 0) || squash_merged,
                squash_merged,
                stale: commit.time().seconds() < stale_cutoff,
                upstream,
                upstream_gone,
            });
        }

        branches.sort_by(|a, b| {
            a.status()
                .cmp(&b.status())
                .then(a.last_commit.time_seconds.cmp(&b.last_commit.time_seconds))
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(Self {
            base: base_ref.map(Self::short_name),
            branches,
        })
    }

    // prefer the remote's default branch since it reflects what the team
    // has merged, then a local main/master, then whatever is checked out
    fn find_base(repo: &KitRepo) -> Option<(String, Oid)> {
        let resolve = |name: &str| {
            let reference = repo.inner.find_reference(name).ok()?;
            let oid = reference.peel_to_commit().ok()?.id();
            Some((reference.name().ok()?.to_string(), oid))
        };

        let remote_default = repo
            .inner
            .find_reference("refs/remotes/origin/HEAD")
            .ok()
            .and_then(|r| r.symbolic_target().ok().flatten().map(str::to_string));

        remote_default
            .into_iter()
            .chain(BASE_CANDIDATES.map(|name| format!("refs/heads/{name}")))
            .find_map(|name| resolve(&name))
            .or_else(|| {
                let head = repo.inner.head().ok()?;
                // a detached head has no branch to exclude, name it anyway
                let name = head.name().unwrap_or("HEAD").to_string();
                Some((name, head.peel_to_commit().ok()?.id()))
            })
    }

    // (upstream short name, whether the upstream ref no longer exists)
    fn upstream(repo: &KitRepo, ref_name: &str) -> (Option<String>, bool) {
        // reads the branch config only, so it works even if the ref is gone
        let Ok(buf) = repo.inner.branch_upstream_name(ref_name) else {
            return (None, false);
        };
        let Ok(upstream) = buf.as_str() else {
            return (None, false);
        };
        let gone = repo.inner.find_reference(upstream).is_err();
        (Some(Self::short_name(upstream)), gone)
    }

    fn short_name(ref_name: &str) -> String {
        ["refs/heads/", "refs/remotes/"]
            .iter()
            .find_map(|prefix| ref_name.strip_prefix(prefix))
            .unwrap_or(ref_name)
            .to_string()
    }
}

// memoised patch ids of base commits, shared by every branch
#[derive(Default)]
struct PatchIds(HashMap<Oid, Option<Oid>>);

impl PatchIds {
    fn get(&mut self, repo: &Repository, oid: Oid) -> Option<Oid> {
        *self.0.entry(oid).or_insert_with(|| {
            let commit = repo.find_commit(oid).ok()?;
            if commit.parent_count() != 1 {
                return None;
            }
            let parent_tree = commit.parent(0).ok()?.tree().ok()?;
            tree_patch_id(repo, &parent_tree, &commit.tree().ok()?)
        })
    }
}

fn tree_patch_id(repo: &Repository, old: &Tree, new: &Tree) -> Option<Oid> {
    let diff = repo.diff_tree_to_tree(Some(old), Some(new), None).ok()?;
    if diff.deltas().len() == 0 {
        return None;
    }
    diff.patchid(None).ok()
}

// a squash merge lands the branch's whole change as one commit on base, so
// compare the branch's combined diff with each base commit since the fork
// (same idea as `git cherry`)
fn is_squash_merged(repo: &Repository, tip: Oid, base: Oid, patch_ids: &mut PatchIds) -> bool {
    let Ok(fork) = repo.merge_base(tip, base) else {
        return false;
    };
    let trees = repo
        .find_commit(fork)
        .and_then(|c| c.tree())
        .and_then(|fork_tree| Ok((fork_tree, repo.find_commit(tip)?.tree()?)));
    let Ok((fork_tree, tip_tree)) = trees else {
        return false;
    };
    let Some(branch_id) = tree_patch_id(repo, &fork_tree, &tip_tree) else {
        return false;
    };

    let Ok(mut walk) = repo.revwalk() else {
        return false;
    };
    if walk.push(base).is_err() || walk.hide(fork).is_err() {
        return false;
    }
    walk.flatten()
        .take(MAX_SQUASH_SCAN)
        .any(|oid| patch_ids.get(repo, oid) == Some(branch_id))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use git2::{Repository, Signature, Time};

    use super::*;

    const DAY: i64 = 24 * 60 * 60;
    const NOW: i64 = 1_800_000_000;

    struct TempRepo {
        path: PathBuf,
        repo: Repository,
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    impl TempRepo {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("gitkit-branches-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            let mut opts = git2::RepositoryInitOptions::new();
            opts.initial_head("main");
            let repo = Repository::init_opts(&path, &opts).unwrap();
            Self { path, repo }
        }

        // empty-tree commit on `refname` at `time`, returns its oid
        fn commit(&self, refname: &str, parent: Option<Oid>, time: i64) -> Oid {
            let sig = Signature::new("dev", "dev@example.com", &Time::new(time, 0)).unwrap();
            let tree_oid = self.repo.treebuilder(None).unwrap().write().unwrap();
            let tree = self.repo.find_tree(tree_oid).unwrap();
            let parents: Vec<_> = parent
                .map(|p| self.repo.find_commit(p).unwrap())
                .into_iter()
                .collect();
            let parents: Vec<_> = parents.iter().collect();
            self.repo
                .commit(Some(refname), &sig, &sig, "msg", &tree, &parents)
                .unwrap()
        }

        // commit on top of `parent` with `path` set to `content`
        fn commit_file(
            &self,
            refname: &str,
            parent: Oid,
            path: &str,
            content: &str,
            time: i64,
        ) -> Oid {
            let sig = Signature::new("dev", "dev@example.com", &Time::new(time, 0)).unwrap();
            let parent = self.repo.find_commit(parent).unwrap();
            let blob = self.repo.blob(content.as_bytes()).unwrap();
            let mut builder = self
                .repo
                .treebuilder(Some(&parent.tree().unwrap()))
                .unwrap();
            builder.insert(path, blob, 0o100644).unwrap();
            let tree = self.repo.find_tree(builder.write().unwrap()).unwrap();
            self.repo
                .commit(Some(refname), &sig, &sig, "msg", &tree, &[&parent])
                .unwrap()
        }

        fn data(&self) -> BranchData {
            let kit = KitRepo::open(&self.path).unwrap();
            BranchData::at(&kit, DateTime::from_timestamp(NOW, 0).unwrap()).unwrap()
        }
    }

    fn find<'a>(data: &'a BranchData, name: &str) -> &'a BranchInfo {
        data.branches.iter().find(|b| b.name == name).unwrap()
    }

    #[test]
    fn classifies_branches_against_base() {
        let t = TempRepo::new("classify");
        let root = t.commit("refs/heads/main", None, NOW - 200 * DAY);
        let tip = t.commit("refs/heads/main", Some(root), NOW - DAY);

        // points at an ancestor of main
        t.repo.reference("refs/heads/done", root, true, "").unwrap();
        // own commit, recent
        t.commit("refs/heads/wip", Some(tip), NOW - 2 * DAY);
        // own commit, old
        t.commit("refs/heads/abandoned", Some(root), NOW - 120 * DAY);

        let data = t.data();
        assert_eq!(data.base.as_deref(), Some("main"));
        assert!(data.branches.iter().all(|b| b.name != "main"));

        let done = find(&data, "done");
        assert_eq!(done.status(), BranchStatus::Merged);
        assert_eq!((done.ahead, done.behind), (0, 1));

        let wip = find(&data, "wip");
        assert_eq!(wip.status(), BranchStatus::Active);
        assert_eq!((wip.ahead, wip.behind), (1, 0));

        let abandoned = find(&data, "abandoned");
        assert_eq!(abandoned.status(), BranchStatus::Stale);
        assert_eq!((abandoned.ahead, abandoned.behind), (1, 1));

        let order: Vec<_> = data.branches.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(order, ["done", "abandoned", "wip"]);
    }

    #[test]
    fn detects_squash_merges() {
        let t = TempRepo::new("squash");
        let root = t.commit("refs/heads/main", None, NOW - 10 * DAY);

        // two commits on the branch, then the same change as one commit on main
        let one = t.commit_file("refs/heads/feat", root, "a.txt", "1\n", NOW - 5 * DAY);
        t.commit_file("refs/heads/feat", one, "a.txt", "1\n2\n", NOW - 4 * DAY);
        let other = t.commit_file("refs/heads/main", root, "b.txt", "x\n", NOW - 3 * DAY);
        t.commit_file("refs/heads/main", other, "a.txt", "1\n2\n", NOW - 2 * DAY);
        // touches the same file but differently, not merged
        t.commit_file("refs/heads/other", root, "a.txt", "1\n3\n", NOW - DAY);

        let data = t.data();

        let feat = find(&data, "feat");
        assert!(feat.squash_merged);
        assert_eq!(feat.status(), BranchStatus::Merged);
        assert_eq!((feat.ahead, feat.behind), (2, 2));

        let other = find(&data, "other");
        assert!(!other.squash_merged);
        assert_eq!(other.status(), BranchStatus::Active);
    }

    #[test]
    fn flags_upstream_that_was_deleted() {
        let t = TempRepo::new("gone");
        let root = t.commit("refs/heads/main", None, NOW - DAY);
        t.commit("refs/heads/feat", Some(root), NOW - DAY);
        t.commit("refs/heads/kept", Some(root), NOW - DAY);

        t.repo
            .remote("origin", "https://example.invalid/repo.git")
            .unwrap();
        let mut config = t.repo.config().unwrap();
        for name in ["feat", "kept"] {
            config
                .set_str(&format!("branch.{name}.remote"), "origin")
                .unwrap();
            config
                .set_str(
                    &format!("branch.{name}.merge"),
                    &format!("refs/heads/{name}"),
                )
                .unwrap();
        }
        // only `kept` still exists on the remote
        let kept = t.repo.refname_to_id("refs/heads/kept").unwrap();
        t.repo
            .reference("refs/remotes/origin/kept", kept, true, "")
            .unwrap();

        let data = t.data();

        let feat = find(&data, "feat");
        assert_eq!(feat.upstream.as_deref(), Some("origin/feat"));
        assert!(feat.upstream_gone);
        assert_eq!(feat.status(), BranchStatus::Gone);

        let kept = find(&data, "kept");
        assert_eq!(kept.upstream.as_deref(), Some("origin/kept"));
        assert!(!kept.upstream_gone);
        assert_eq!(kept.status(), BranchStatus::Active);

        let remote = find(&data, "origin/kept");
        assert_eq!(remote.kind, BranchKind::Remote);
    }
}
