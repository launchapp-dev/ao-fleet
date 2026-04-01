use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Import hosts and projects from a remote hostd instance")]
pub struct HostImportCommand {
    #[arg(long)]
    pub base_url: String,

    #[arg(long)]
    pub auth_token: Option<String>,

    #[arg(long, default_value_t = false)]
    pub register_projects: bool,

    #[arg(long)]
    pub team_id: Option<String>,

    #[arg(long, default_value = "hostd")]
    pub assignment_source: String,
}
