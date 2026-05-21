mod args;
mod build;
mod commands;
mod source;

use anyhow::Result;
use clap::Parser;

use crate::args::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(cli)
}
