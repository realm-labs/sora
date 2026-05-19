mod args;
mod commands;

use anyhow::Result;
use clap::Parser;

use crate::args::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(cli.command)
}
