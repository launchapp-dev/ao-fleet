use clap::Args;

#[derive(Debug, Args)]
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
