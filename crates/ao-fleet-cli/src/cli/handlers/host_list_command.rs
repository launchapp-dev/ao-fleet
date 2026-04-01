use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List all registered hosts")]
pub struct HostListCommand;
