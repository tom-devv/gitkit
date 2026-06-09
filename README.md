# GitKit

GitKit provides a sleek TUI (terminal user interface) to display insightful metrics about your team's git repository.


## Demo

<img width="812" height="807" alt="image" src="https://github.com/user-attachments/assets/79b4f993-0d7c-41dc-84a9-6396bd991e4a" />


<img width="479" height="594" alt="image" src="https://github.com/user-attachments/assets/99e61531-ed12-4884-a76d-8f703f786916" />



## Features 

🌎 Overivew - General overview of the repository including: status, last activity, total commits
📈 Cadence - Repository and per user cadence patterns: commits per week, repo share %, first commit
🚨 Silo - Assess the risk of a knowledge silo per file via churn: Silo risk %, total churn 

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

