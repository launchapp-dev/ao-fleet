use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Delete a daemon schedule")]
pub struct ScheduleDeleteCommand {
    #[arg(long)]
    pub id: String,
}
