mod args;
mod build;
mod commands;
mod init;
mod lua_parser;
mod report;
mod source;
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
