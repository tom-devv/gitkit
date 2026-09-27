[![Crates.io](https://img.shields.io/crates/v/gitkit-cli.svg)](https://crates.io/crates/gitkit-cli)
[![License](https://img.shields.io/crates/l/gitkit-cli.svg)](https://github.com/tom-devv/gitkit)
# GitKit

GitKit is a fast, terminal-based repository explorer. It visualizes developer behavior, tracks code churn over time, and helps identify knowledge bottlenecks directly from your command line.

## Demo

<div align="center">


<img src="https://raw.githubusercontent.com/tom-devv/gitkit/main/assets/gitkit_demo.gif" alt="GitKit Demo" width="100%" />
</div>


## Features 

🌎 Overview - General overview of the repository including: status, last activity, total commits

📈 Cadence - Repository and per user cadence patterns: commits per week, repo share %, first commit

🚨 Silo - Assess the risk of a knowledge silo per file via churn: Silo risk %, total churn

🌿 Branches - Branch hygiene against the default branch: merged (including squash merges), stale, upstream gone, ahead/behind


### Prerequisites
-  [Rust / Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (for installation)

## Installation

You can use cargo to install gitkit:

```shell
cargo install gitkit-cli
```

You can find compiled binaries on the [Github Releases page](https://github.com/tom-devv/gitkit/releases/latest) for:


- [macOS Silicon](https://github.com/tom-devv/gitkit/releases/latest/download/gitkit-aarch64-apple-darwin.tar.gz)


- [macOS Intel](https://github.com/tom-devv/gitkit/releases/latest/download/gitkit-x86_64-apple-darwin.tar.gz)

- [Linux x86-64](https://github.com/tom-devv/gitkit/releases/latest/download/gitkit-x86_64-unknown-linux-musl.tar.gz)

- [Windows x86-64](https://github.com/tom-devv/gitkit/releases/latest/download/gitkit-x86_64-pc-windows-msvc.zip)

    - _Note: double clicking the executable search for a git repo in the directory where it was opened. se cmd to specify a directory_



## Usage

Usage is simple:

```shell
gitkit [TARGET_PATH]
```

`[TARGET_PATH]` is an optional argument and defaults to the current directory if not specified

### JSON output

Use `--json` to print the metrics to stdout instead of opening the TUI, e.g. for scripts or CI:

```shell
gitkit --json [TARGET_PATH]
```

Add `--only` with a comma separated list of `home`, `cadence`, `silo` and `branches` to only compute those sections. Skipping `silo` is much faster on large repositories, since it diffs the whole history:

```shell
gitkit --json --only home,cadence
```

The output looks like this (arrays shortened):

```json
{
  "gitkit_version": "0.1.5",
  "home": {
    "repo_name": "gitkit",
    "current_branch": "main",
    "total_commits": 1823,
    "first_commit": { "id": "5045…", "author_email": "dev@example.com", "date": "2025-09-30T15:17:29Z", "timestamp": 1759245449 },
    "last_commit": { "id": "7be0…", "author_email": "dev@example.com", "date": "2026-04-28T19:44:01Z", "timestamp": 1777405441 },
    "status": [{ "path": "src/main.rs", "index": null, "worktree": "modified" }]
  },
  "cadence": {
    "global_commits_per_week": 60.7,
    "authors": [{
      "email": "dev@example.com",
      "total_commits": 532,
      "commits_per_week": 17.7,
      "repo_share_percent": 29.2,
      "first_commit": "2025-09-30T15:17:29Z",
      "activity": { "mon": [0, 0, "…24 hourly counts (UTC)"], "tue": [], "wed": [], "thu": [], "fri": [], "sat": [], "sun": [] }
    }]
  },
  "silo": {
    "files": [{
      "path": "src/main.rs",
      "gatekeeper": "dev@example.com",
      "contributors": 1,
      "risk_percent": 100,
      "total_churn": 576,
      "authors": [{ "email": "dev@example.com", "churn": 576 }]
    }]
  },
  "branches": {
    "base": "origin/main",
    "stale_after_days": 90,
    "branches": [{
      "name": "fix/typo",
      "kind": "local",
      "is_head": false,
      "status": "merged",
      "merged": true,
      "squash_merged": true,
      "stale": false,
      "ahead": 1,
      "behind": 6,
      "upstream": "origin/fix/typo",
      "upstream_gone": true,
      "last_commit": { "id": "629f…", "author_email": "dev@example.com", "date": "2026-07-19T13:50:25Z", "timestamp": 1784469025 }
    }]
  }
}
```

Authors, files and branches are sorted the same way as in the TUI: authors by commits per week, files by silo risk and then churn, and branches by status (`merged`, `gone`, `stale`, `active`) and then oldest first.

Branches are compared against the remote's default branch (`origin/HEAD`), falling back to a local `main` or `master`, then the checked out branch. A branch is `merged` when its tip is in that base, or when its combined change matches a single commit on it (a squash merge).

For example, to list files where one person wrote more than 90% of the churn:

```shell
gitkit --json --only silo | jq -r '.silo.files[] | select(.risk_percent > 90) | .path'
```

Or to list local branches that are merged or whose upstream was deleted:

```shell
gitkit --json --only branches | jq -r '.branches.branches[] | select(.kind == "local" and (.status == "merged" or .status == "gone")) | .name'
```

## Keybindings

GitKit is built for fast keyboard navigation:
| Key | Action |
| :--- | :--- |
| `Tab` | Cycle through pages (Overview, Cadence, Silo, Branches) |
| `j` / `k` | Scroll up and down through lists |
| `/` | Open the search modal to filter the current view |
| `Esc` | Clear search or close modals |
| `q` | Quit the application |

## Contributing

Contributions, issues, and feature requests are welcome.

## License

Distributed under the MIT License. See `LICENSE` for more information.