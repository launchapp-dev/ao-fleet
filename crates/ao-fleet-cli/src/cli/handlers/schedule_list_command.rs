use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List daemon schedules, optionally filtered by team")]
pub struct ScheduleListCommand {
    #[arg(long)]
    pub team_id: Option<String>,
}
