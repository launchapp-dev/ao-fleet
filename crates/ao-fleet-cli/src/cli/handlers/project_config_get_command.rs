use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct ProjectConfigGetCommand {
    #[arg(long)]
    pub project_root: String,
}
