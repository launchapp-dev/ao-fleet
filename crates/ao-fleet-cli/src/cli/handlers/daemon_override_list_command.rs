use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List active daemon schedule overrides")]
pub struct DaemonOverrideListCommand;
