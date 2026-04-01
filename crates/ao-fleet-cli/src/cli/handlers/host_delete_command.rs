use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Remove a host from the fleet registry")]
pub struct HostDeleteCommand {
    #[arg(long)]
    pub id: String,
}
