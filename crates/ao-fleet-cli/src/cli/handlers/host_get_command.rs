use clap::Args;

#[derive(Debug, Args)]
pub struct HostGetCommand {
    #[arg(long)]
    pub id: String,
}
