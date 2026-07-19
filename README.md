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