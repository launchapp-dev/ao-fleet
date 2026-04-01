use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Show a founder-level summary of team activity and project status")]
pub struct FounderOverviewCommand {
    #[arg(long)]
    pub team_id: Option<String>,
}
