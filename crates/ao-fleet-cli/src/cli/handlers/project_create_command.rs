use clap::Args;

#[derive(Debug, Args)]
pub struct ProjectCreateCommand {
    #[arg(long)]
    pub team_id: String,

    #[arg(long)]
    pub slug: String,

    #[arg(long)]
    pub root_path: String,

    #[arg(long)]
    pub ao_project_root: String,

    #[arg(long, default_value = "main")]
    pub default_branch: String,

    #[arg(long)]
    pub remote_url: Option<String>,

    #[arg(long, default_value_t = true)]
    pub enabled: bool,
}
