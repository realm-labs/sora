mod args;
mod build;
mod commands;
mod init;
mod mcp;
mod report;
mod studio;

use clap::Parser;

use crate::args::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(error) = commands::run(cli) {
        eprintln!("{}", report::ErrorReport::new(&error));
        std::process::exit(1);
    }
}
