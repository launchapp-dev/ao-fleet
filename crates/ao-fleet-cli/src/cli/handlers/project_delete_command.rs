use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Remove a project from the fleet registry")]
pub struct ProjectDeleteCommand {
    #[arg(long)]
    pub id: String,
}
