use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Fetch recent logs from a remote host")]
pub struct HostLogsCommand {
    #[arg(long)]
    pub base_url: String,

    #[arg(long)]
    pub auth_token: Option<String>,

    #[arg(long)]
    pub project_id: Option<String>,

    #[arg(long)]
    pub after_seq: Option<u64>,

    #[arg(long, default_value_t = 200)]
    pub limit: usize,

    #[arg(long)]
    pub cat: Option<String>,

    #[arg(long)]
    pub level: Option<String>,

    #[arg(long)]
    pub workflow: Option<String>,

    #[arg(long)]
    pub run: Option<String>,
}
