use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Get details for a specific host")]
pub struct HostGetCommand {
    #[arg(long)]
    pub id: String,
}
