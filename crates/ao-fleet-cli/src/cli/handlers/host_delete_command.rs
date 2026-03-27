use clap::Args;

#[derive(Debug, Args)]
pub struct HostDeleteCommand {
    #[arg(long)]
    pub id: String,
}
