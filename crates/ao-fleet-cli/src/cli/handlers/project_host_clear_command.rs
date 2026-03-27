use clap::Args;

#[derive(Debug, Args)]
pub struct ProjectHostClearCommand {
    #[arg(long)]
    pub project_id: String,
}
