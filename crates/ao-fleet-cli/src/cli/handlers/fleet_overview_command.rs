use clap::Args;

#[derive(Debug, Args)]
pub struct FleetOverviewCommand {
    #[arg(long)]
    pub team_id: Option<String>,
}
