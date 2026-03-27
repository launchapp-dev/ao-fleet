use clap::Args;

#[derive(Debug, Args)]
pub struct ScheduleDeleteCommand {
    #[arg(long)]
    pub id: String,
}
