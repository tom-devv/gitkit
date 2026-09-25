use clap::Parser;
use gitkit_cli::KitArgs;
use std::process;

fn main() {
    let args = KitArgs::parse();

    if let Err(err) = gitkit_cli::run(args) {
        eprintln!("Process failed with: {}", err);
        process::exit(1);
    }
}
