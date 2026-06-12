use std::collections::{HashMap, HashSet};

use git2::{Patch, TreeWalkMode, TreeWalkResult};

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
        let mut churn_map: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for (commit, diff) in repo.iter_diff_history()? {
            let author_name = commit.email;

            // TODO ENSURE NO DIVISION HERE

            for i in 0..diff.deltas().len() {
                if let Ok(Some(patch)) = Patch::from_diff(&diff, i) {
                    if let Some(path) = patch.delta().new_file().path() {
                        let file_path = path.to_string_lossy().to_string();

                        if let Ok((insertions, deletions, _)) = patch.line_stats() {
                            let churn = insertions + deletions;

                            if churn > 0 {
                                *churn_map
                                    .entry(file_path)
                                    .or_default()
                                    .entry(author_name.clone())
                                    .or_default() += churn;
                            }
                        }
                    }
                }
            }
        }

        Ok(churn_map)
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
