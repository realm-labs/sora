mod args;
mod build;
mod commands;
mod init;
mod mcp;
mod report;
mod studio;

use std::process::exit;

use clap::Parser;

use crate::{args::Cli, report::ErrorReport};

fn main() {
    let cli = Cli::parse();
    if let Err(error) = commands::run(cli) {
        eprintln!("{}", ErrorReport::new(&error));
        exit(1);
    }
}
