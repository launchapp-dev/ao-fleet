use clap::Args;

#[derive(Debug, Args)]
#[command(
    about = "Show a health rollup summary across all fleet daemons",
    long_about = "Aggregates observed vs desired daemon state for every project in the fleet. \
Useful for a quick health check: how many daemons are aligned, degraded, or unobserved."
)]
pub struct DaemonHealthRollupCommand {
    #[arg(long, help = "Filter to a specific team ID")]
    pub team_id: Option<String>,
}
