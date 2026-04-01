use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Show a full overview of fleet health and daemon status")]
pub struct FleetOverviewCommand {
    #[arg(long)]
    pub team_id: Option<String>,
}
