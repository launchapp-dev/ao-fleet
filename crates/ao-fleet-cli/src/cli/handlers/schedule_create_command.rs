use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Create a daemon activity schedule for a team")]
pub struct ScheduleCreateCommand {
    #[arg(long)]
    pub team_id: String,

    #[arg(long)]
    pub timezone: String,

    #[arg(long)]
    pub policy_kind: String,

    #[arg(long = "window", value_name = "weekday,start,end")]
    pub windows: Vec<String>,

    #[arg(long, default_value_t = true)]
    pub enabled: bool,
}
