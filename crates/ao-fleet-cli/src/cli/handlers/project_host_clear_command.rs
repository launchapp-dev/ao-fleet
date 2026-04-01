use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Clear the host assignment for a project")]
pub struct ProjectHostClearCommand {
    #[arg(long)]
    pub project_id: String,
}
