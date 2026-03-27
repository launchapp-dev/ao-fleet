mod cli;

use anyhow::Result;
use clap::Parser;

use crate::cli::root_command::RootCommand;
use crate::cli::run::run;

fn main() -> Result<()> {
    run(RootCommand::parse())
}
