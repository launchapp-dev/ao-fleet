use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List all teams registered in the fleet")]
pub struct TeamListCommand;
