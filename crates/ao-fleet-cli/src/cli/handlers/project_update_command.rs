use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Update fields on a registered project")]
pub struct ProjectUpdateCommand {
    #[arg(long)]
    pub id: String,

    #[arg(long)]
    pub team_id: Option<String>,

    #[arg(long)]
    pub slug: Option<String>,

    #[arg(long)]
    pub root_path: Option<String>,

    #[arg(long)]
    pub ao_project_root: Option<String>,

    #[arg(long)]
    pub default_branch: Option<String>,

    #[arg(long)]
    pub remote_url: Option<String>,

    #[arg(long)]
    pub enabled: Option<bool>,
}
