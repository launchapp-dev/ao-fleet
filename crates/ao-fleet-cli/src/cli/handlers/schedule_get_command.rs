use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Get details for a specific daemon schedule")]
pub struct ScheduleGetCommand {
    #[arg(long)]
    pub id: String,
}
