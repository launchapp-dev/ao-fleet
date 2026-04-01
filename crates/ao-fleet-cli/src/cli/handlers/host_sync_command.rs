use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Sync project registrations from a specific remote host")]
pub struct HostSyncCommand {
    #[arg(long)]
    pub base_url: String,

    #[arg(long)]
    pub auth_token: Option<String>,

    #[arg(long, default_value_t = false)]
    pub register_missing: bool,

    #[arg(long)]
    pub team_id: Option<String>,

    #[arg(long, default_value = "hostd_sync")]
    pub assignment_source: String,
}
