use git2::DiffStats;

use super::kit::KitRepo;

#[derive(Debug, Default)]
pub struct Contribution {
    pub commits: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl Contribution {
    pub fn add_stats(&mut self, stats: DiffStats) {
        self.commits += 1;
        self.insertions += stats.insertions();
        self.deletions += stats.deletions();
    }
}

impl KitRepo {
    pub fn get_entire_repo_contribution(&self) -> Result<Contribution, git2::Error> {
        let mut contrib = Contribution::default();
        for commit in self.iter_raw_commits()? {
            let diff = self.get_parent_diff(&commit, None)?; // TODO add opts argument
            let stats = diff.stats()?;

            contrib.add_stats(stats);
        }
        Ok(contrib)
    }

    pub fn get_lifetime_contributions(&self, email: String) -> Result<Contribution, git2::Error> {
        let mut contrib = Contribution::default();
        for commit in self.get_raw_author_commits(&email)? {
            let diff = self.get_parent_diff(&commit, None)?; // TODO add opts argument
            let stats = diff.stats()?;
            contrib.add_stats(stats);
        }

        Ok(contrib)
    }
}
