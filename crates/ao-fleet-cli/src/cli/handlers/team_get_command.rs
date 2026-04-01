use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Get details for a specific team")]
pub struct TeamGetCommand {
    #[arg(long)]
    pub id: String,
}
