use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Get details for a specific project")]
pub struct ProjectGetCommand {
    #[arg(long)]
    pub id: String,
}
