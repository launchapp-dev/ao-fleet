use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List all project-to-host assignments")]
pub struct ProjectHostListCommand;
