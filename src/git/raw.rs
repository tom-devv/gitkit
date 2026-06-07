use git2::Commit;

use crate::git::kit::KitRepo;

impl KitRepo {
    pub(in crate::git) fn iter_raw_commits(
        &self,
    ) -> Result<impl Iterator<Item = git2::Commit<'_>>, git2::Error> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.push_head()?;

        let repo_ref = &self.inner;
        Ok(revwalk
            .flatten()
            .filter_map(move |oid| repo_ref.find_commit(oid).ok()))
    }

    pub(in crate::git) fn get_raw_author_commits(
        &self,
        email: &str,
    ) -> Result<impl Iterator<Item = Commit<'_>>, git2::Error> {
        let email_owned = email.to_string();
        let iter = self
            .iter_raw_commits()?
            .filter(move |commit| commit.author().email().map_or(false, |e| e == email_owned));
        Ok(iter)
    }
}
