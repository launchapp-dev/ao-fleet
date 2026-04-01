use clap::Args;

#[derive(Debug, Clone, Args)]
#[command(about = "Get the AO runtime config for a project")]
pub struct ProjectConfigGetCommand {
    #[arg(long)]
    pub project_root: String,
}
