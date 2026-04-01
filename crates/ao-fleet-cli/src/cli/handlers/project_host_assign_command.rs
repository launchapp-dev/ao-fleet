use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Assign a project to a specific execution host")]
pub struct ProjectHostAssignCommand {
    #[arg(long)]
    pub project_id: String,

    #[arg(long)]
    pub host_id: String,

    #[arg(long, default_value = "manual")]
    pub assignment_source: String,
}
