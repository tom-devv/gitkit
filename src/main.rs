use clap::Parser;
use gitkit_cli::KitArgs;
use std::process;

fn main() {
    let args = KitArgs::parse();

    match gitkit_cli::run(args) {
        Ok(_) => process::exit(1),
        Err(err) => eprintln!("Process failed with: {}", err),
    }
}
