[![Crates.io](https://img.shields.io/crates/v/gitkit-cli.svg)](https://crates.io/crates/gitkit-cli)
[![License](https://img.shields.io/crates/l/gitkit-cli.svg)](https://github.com/tom-devv/gitkit)
# GitKit

GitKit is a fast, terminal-based repository explorer. It visualizes developer behavior, tracks code churn over time, and helps identify knowledge bottlenecks directly from your command line.

## Demo

<div align="center">


   <img width="1920" height="978" src="./assets/gitkit_demo.gif" alt="GitKit Demo"      style="max-width: 100%; height: auto;" />
</div>


## Features 

🌎 Overview - General overview of the repository including: status, last activity, total commits

📈 Cadence - Repository and per user cadence patterns: commits per week, repo share %, first commit

🚨 Silo - Assess the risk of a knowledge silo per file via churn: Silo risk %, total churn


### Prerequisites
-  [Rust / Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (for installation)

## Installation

You can use cargo to install gitkit:

```shell
cargo install gitkit-cli
```

## Usage

Usage is simple:

```shell
gitkit [TARGET_PATH]
```

`[TARGET_PATH]` is an optional argument and defaults to the current directory if not specified

## Keybindings

GitKit is built for fast keyboard navigation:
| Key | Action |
| :--- | :--- |
| `Tab` | Cycle through pages (Overview, Cadence, Silo) |
| `j` / `k` | Scroll up and down through lists |
| `/` | Open the search modal to filter the current view |
| `Esc` | Clear search or close modals |
| `q` | Quit the application |

## Contributing

Contributions, issues, and feature requests are welcome.

## License

Distributed under the MIT License. See `LICENSE` for more information.