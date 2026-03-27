use anyhow::Result;

use crate::cli::command_router::route_command;
use crate::cli::root_command::RootCommand;

pub fn run(command: RootCommand) -> Result<()> {
    route_command(command)
}
