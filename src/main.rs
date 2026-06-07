use clap::Parser;
use gitkit_cli::GKitArgs;
use std::process;

fn main() {
    let args = GKitArgs::parse();

    match gitkit_cli::run(args) {
        Ok(_) => process::exit(1),
        Err(err) => eprintln!("Process failed with: {}", err),
    }
}
