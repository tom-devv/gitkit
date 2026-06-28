use std::collections::{HashMap, HashSet};

use git2::{DiffOptions, Oid, Patch, Repository, TreeWalkMode, TreeWalkResult};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::error::Result;
use crate::git::kit::KitRepo;

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
        let commit_pairs = Self::extract_commit_oids(repo)?;
        let repo_path = repo.inner.path().to_path_buf();

        let merged_churn_map = commit_pairs
            .par_iter()
            .fold(
                || HashMap::new(),
                |mut local_map: HashMap<String, HashMap<String, usize>>,
                 (commit_oid, parent_oid, author_email)| {
                    if let Ok(local_repo) = KitRepo::open(&repo_path) {
                        let mut diff_opts = DiffOptions::new();
                        diff_opts
                            .skip_binary_check(true)
                            .ignore_filemode(true)
                            .ignore_submodules(true)
                            .enable_fast_untracked_dirs(true);

                        if let Ok(diff) = local_repo.get_diff_from_oids(
                            Some(*parent_oid),
                            *commit_oid,
                            Some(&mut diff_opts),
                        ) {
                            for i in 0..diff.deltas().len() {
                                if let Ok(Some(patch)) = Patch::from_diff(&diff, i) {
                                    if let Some(path) = patch.delta().new_file().path() {
                                        let file_path = path.to_string_lossy().to_string();

                                        if let Ok((insertions, deletions, _)) = patch.line_stats() {
                                            let churn = insertions + deletions;
                                            if churn > 0 {
                                                *local_map
                                                    .entry(file_path)
                                                    .or_default()
                                                    .entry(author_email.clone())
                                                    .or_default() += churn;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    local_map
                },
            )
            .reduce(
                || HashMap::new(),
                |mut map_a, map_b| {
                    for (file, authors_b) in map_b {
                        let authors_a = map_a.entry(file).or_default();
                        for (author, churn) in authors_b {
                            *authors_a.entry(author).or_default() += churn;
                        }
                    }
                    map_a
                },
            );

        Ok(merged_churn_map)
    }

    fn extract_commit_oids(repo: &KitRepo) -> Result<Vec<(Oid, Oid, String)>> {
        let mut pairs = Vec::new();

        let mut revwalk = repo.inner.revwalk()?;
        revwalk.push_head()?;

        for oid_result in revwalk {
            if let Ok(oid) = oid_result {
                if let Ok(commit) = repo.inner.find_commit(oid) {
                    if commit.parent_count() == 1 {
                        if let Ok(parent) = commit.parent(0) {
                            let email = commit.author().email().unwrap_or("Unknown").to_string();
                            pairs.push((commit.id(), parent.id(), email));
                        }
                    }
                }
            }
        }
        Ok(pairs)
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
