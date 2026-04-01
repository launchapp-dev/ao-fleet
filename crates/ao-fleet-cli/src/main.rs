use anyhow::Result;
use ao_fleet_cli::cli::root_command::RootCommand;
use ao_fleet_cli::cli::run::run;
use clap::Parser;

fn main() -> Result<()> {
    run(RootCommand::parse())
}

