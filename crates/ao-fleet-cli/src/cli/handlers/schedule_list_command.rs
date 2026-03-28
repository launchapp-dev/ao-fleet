use clap::Args;

#[derive(Debug, Args)]
pub struct ScheduleListCommand {
    #[arg(long)]
    pub team_id: Option<String>,
}
