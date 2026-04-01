use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List all projects registered in the fleet")]
pub struct ProjectListCommand;
