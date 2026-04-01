use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Update fields on an existing daemon schedule")]
pub struct ScheduleUpdateCommand {
    #[arg(long)]
    pub id: String,

    #[arg(long)]
    pub team_id: Option<String>,

    #[arg(long)]
    pub timezone: Option<String>,

    #[arg(long)]
    pub policy_kind: Option<String>,

    #[arg(long = "window", value_name = "weekday,start,end")]
    pub windows: Vec<String>,

    #[arg(long)]
    pub enabled: Option<bool>,
}
