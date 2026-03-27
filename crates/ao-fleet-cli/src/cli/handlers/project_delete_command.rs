use clap::Args;

#[derive(Debug, Args)]
pub struct ProjectDeleteCommand {
    #[arg(long)]
    pub id: String,
}
