use clap::Args;

#[derive(Debug, Args)]
pub struct FounderOverviewCommand {
    #[arg(long)]
    pub team_id: Option<String>,
}
