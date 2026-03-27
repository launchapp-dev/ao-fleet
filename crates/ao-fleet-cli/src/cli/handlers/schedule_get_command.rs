use clap::Args;

#[derive(Debug, Args)]
pub struct ScheduleGetCommand {
    #[arg(long)]
    pub id: String,
}
