use std::collections::{HashMap, HashSet};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use git2::{DiffOptions, Oid, Patch, Repository, TreeWalkMode, TreeWalkResult};

use crate::error::Result;
use crate::git::kit::KitRepo;

// libgit2 pack access contends past a handful of threads
const MAX_CHURN_WORKERS: usize = 8;

// blobs bigger than this are almost always binaries or generated/vendored output
const MAX_DIFF_BLOB_SIZE: i64 = 4 * 1024 * 1024;

#[derive(Default)]
pub struct SiloData {
    pub files: Vec<FileSilo>,
}

#[derive(Default, Debug)]
pub struct FileSilo {
    pub file: String,
    pub gatekeeper: String,
    pub contributors: u16,
    pub risk: u8,
    pub total_churn: usize,
    pub author_churn: HashMap<String, usize>,
}

impl SiloData {
    pub fn new(repo: &KitRepo) -> Self {
        SiloData::get_churn(repo).unwrap_or_default()
    }

    pub fn get_churn(repo: &KitRepo) -> Result<Self> {
        let head_files = Self::get_head_files(repo)?;

        let raw_churn_map = Self::accumulate_churn(repo)?;

        let active_files = Self::process_silos(raw_churn_map, &head_files);

        Ok(Self {
            files: active_files,
        })
    }

    pub fn get_head_files(repo: &KitRepo) -> Result<HashSet<String>> {
        let mut current_files = HashSet::new();
        let head = repo.inner.head()?;
        let head_tree = head.peel_to_tree()?;

        head_tree.walk(TreeWalkMode::PreOrder, |root, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                if let Some(name) = entry.name().ok() {
                    current_files.insert(format!("{}{}", root, name));
                }
            }
            TreeWalkResult::Ok
        })?;

        Ok(current_files)
    }

    pub fn accumulate_churn(repo: &KitRepo) -> Result<HashMap<String, HashMap<String, usize>>> {
        let (commit_pairs, authors) = Self::extract_tree_pairs(repo)?;
        let repo_path = repo.inner.path();

        let workers = thread::available_parallelism()
            .map_or(1, |n| n.get())
            .min(MAX_CHURN_WORKERS)
            .min(commit_pairs.len().max(1));
        let next = AtomicUsize::new(0);

        // each worker keeps one repo open (and its object cache warm) and pulls
        // commits off a shared counter, so big commits don't stall a fixed chunk
        let local_maps: Vec<HashMap<String, HashMap<usize, usize>>> = thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(|| {
                        let mut local_map: HashMap<String, HashMap<usize, usize>> = HashMap::new();
                        let Ok(local_repo) = Repository::open(repo_path) else {
                            return local_map;
                        };

                        let mut diff_opts = DiffOptions::new();
                        diff_opts
                            .skip_binary_check(true)
                            .ignore_filemode(true)
                            .ignore_submodules(true)
                            .enable_fast_untracked_dirs(true)
                            // churn only counts +/- lines, context lines are wasted work
                            .context_lines(0)
                            .interhunk_lines(0)
                            // treat huge blobs as binary (0 churn) without inflating them
                            .max_size(MAX_DIFF_BLOB_SIZE);

                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some((tree_oid, parent_tree_oid, author)) = commit_pairs.get(i)
                            else {
                                break;
                            };
                            Self::diff_churn(
                                &local_repo,
                                *parent_tree_oid,
                                *tree_oid,
                                *author,
                                &mut diff_opts,
                                &mut local_map,
                            );
                        }
                        local_map
                    })
                })
                .collect();

            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        });

        let mut merged: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for local_map in local_maps {
            for (file, authors_churn) in local_map {
                let merged_authors = merged.entry(file).or_default();
                for (author, churn) in authors_churn {
                    *merged_authors.entry(authors[author].clone()).or_default() += churn;
                }
            }
        }

        Ok(merged)
    }

    fn diff_churn(
        repo: &Repository,
        parent_tree_oid: Oid,
        tree_oid: Oid,
        author: usize,
        diff_opts: &mut DiffOptions,
        churn_map: &mut HashMap<String, HashMap<usize, usize>>,
    ) {
        let (Ok(parent_tree), Ok(tree)) = (repo.find_tree(parent_tree_oid), repo.find_tree(tree_oid))
        else {
            return;
        };
        let Ok(diff) = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), Some(diff_opts)) else {
            return;
        };

        for i in 0..diff.deltas().len() {
            let Ok(Some(patch)) = Patch::from_diff(&diff, i) else {
                continue;
            };
            let Ok((_context, insertions, deletions)) = patch.line_stats() else {
                continue;
            };
            let churn = insertions + deletions;
            if churn == 0 {
                continue;
            }
            if let Some(path) = patch.delta().new_file().path() {
                let file_path = path.to_string_lossy();
                // avoid allocating a new key for files we've already seen
                let authors_churn = match churn_map.get_mut(file_path.as_ref()) {
                    Some(authors_churn) => authors_churn,
                    None => churn_map.entry(file_path.into_owned()).or_default(),
                };
                *authors_churn.entry(author).or_default() += churn;
            }
        }
    }

    // (commit tree, parent tree, author index) for every non-merge commit,
    // plus the interned author emails the indices point into
    fn extract_tree_pairs(repo: &KitRepo) -> Result<(Vec<(Oid, Oid, usize)>, Vec<String>)> {
        let mut pairs = Vec::new();
        let mut authors: Vec<String> = Vec::new();
        let mut author_ids: HashMap<String, usize> = HashMap::new();

        let mut revwalk = repo.inner.revwalk()?;
        revwalk.push_head()?;

        for oid in revwalk.flatten() {
            let Ok(commit) = repo.inner.find_commit(oid) else {
                continue;
            };
            if commit.parent_count() != 1 {
                continue;
            }
            let Ok(parent) = commit.parent(0) else {
                continue;
            };

            let author = commit.author();
            let email = author.email().unwrap_or("Unknown");
            let author_id = match author_ids.get(email) {
                Some(id) => *id,
                None => {
                    authors.push(email.to_string());
                    author_ids.insert(email.to_string(), authors.len() - 1);
                    authors.len() - 1
                }
            };

            pairs.push((commit.tree_id(), parent.tree_id(), author_id));
        }
        Ok((pairs, authors))
    }

    pub fn process_silos(
        churn_map: HashMap<String, HashMap<String, usize>>,
        head_files: &HashSet<String>,
    ) -> Vec<FileSilo> {
        let mut active_files = Vec::new();

        for (file, author_churn) in churn_map {
            if !head_files.contains(&file) {
                continue;
            }

            let total_churn: usize = author_churn.values().sum();
            let contributors = author_churn.len() as u16;

            let mut gatekeeper = String::from("Unknown");
            let mut top_churn = 0;

            for (author, churn) in &author_churn {
                if *churn > top_churn {
                    top_churn = *churn;
                    gatekeeper = author.clone();
                }
            }

            let risk = if total_churn > 0 {
                ((top_churn as f64 / total_churn as f64) * 100.0).round() as u8
            } else {
                0
            };

            active_files.push(FileSilo {
                file,
                gatekeeper,
                contributors,
                risk,
                total_churn,
                author_churn,
            });
        }

        active_files.sort_by(|a, b| b.risk.cmp(&a.risk).then(b.total_churn.cmp(&a.total_churn)));

        active_files
    }
}
