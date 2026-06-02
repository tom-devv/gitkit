use std::fmt;

use git2::StatusOptions;
use ratatui::{
    style::{
        Color, Stylize,
        palette::material::WHITE,
    },
    text::Line,
};

use crate::git::kit::KitRepo;

#[derive(Debug)]
pub struct KitStatus {
    pub files: Vec<(String, git2::Status)>,
    pub pretty: Vec<(String, String)>,
}

impl KitStatus {
    pub fn new(repo: &KitRepo) -> Self {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let empty = Self {
            files: vec![],
            pretty: vec![("".to_owned(), "no files changes".to_owned())],
        };

        if let Ok(statuses) = repo.inner.statuses(Some(&mut opts)) {
            if statuses.is_empty() {
                return empty;
            }

            let files: Vec<(String, git2::Status)> = statuses
                .iter()
                .map(|entry| {
                    let path = entry.path().unwrap_or("(no path)").to_string();
                    let status = entry.status();
                    (path, status)
                })
                .collect();
            let pretty = KitStatus::pretty(&files);
            Self { files, pretty }
        } else {
            // omit the error silently and return empty vecs
            empty
        }
    }

    // (path, status)
    fn pretty(files: &Vec<(String, git2::Status)>) -> Vec<(String, String)> {
        let mut path_status: Vec<(String, String)> = Vec::new();

        for (path, status) in files {
            let code = match status {
                s if s.contains(git2::Status::WT_NEW) => "??",
                s if s.contains(git2::Status::INDEX_NEW) => "A ",
                s if s.contains(git2::Status::WT_MODIFIED) => " M",
                s if s.contains(git2::Status::INDEX_MODIFIED) => "M ",
                s if s.contains(git2::Status::WT_DELETED) => " D",
                s if s.contains(git2::Status::INDEX_DELETED) => "D ",
                _ => " ",
            };
            path_status.push((path.to_string(), code.to_string()));
        }
        path_status
    }

    // todo add pretty print for empty vec
    pub fn tui_print(&self) -> Vec<Line<'_>> {
        let lines: Vec<Line<'_>> = self
            .pretty
            .iter()
            .map(|(path, code)| {
                let line = Line::from(vec![
                    format!("{}", code).fg(Color::Green),
                    " ".into(),
                    format!(" @ {}", path).fg(WHITE),
                ]);
                line
            })
            .collect();
        lines
    }
}

impl fmt::Display for KitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.files.is_empty() {
            return write!(f, "Your branch is clean.");
        }

        for (path, status) in &self.files {
            // Format the status flags into human-readable shorthand
            let code = match *status {
                s if s.contains(git2::Status::WT_NEW) => "??",
                s if s.contains(git2::Status::INDEX_NEW) => "A ",
                s if s.contains(git2::Status::WT_MODIFIED) => " M",
                s if s.contains(git2::Status::INDEX_MODIFIED) => "M ",
                s if s.contains(git2::Status::WT_DELETED) => " D",
                s if s.contains(git2::Status::INDEX_DELETED) => "D ",
                _ => "  ",
            };
            writeln!(f, "{} {}", code, path)?;
        }
        Ok(())
    }
}
