use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Stream logs from a remote host in real time")]
pub struct HostLogStreamCommand {
    #[arg(long)]
    pub base_url: String,

    #[arg(long)]
    pub auth_token: Option<String>,

    #[arg(long)]
    pub project_id: Option<String>,

    #[arg(long)]
    pub after_seq: Option<u64>,

    #[arg(long)]
    pub tail: Option<usize>,

    #[arg(long)]
    pub cat: Option<String>,

    #[arg(long)]
    pub level: Option<String>,

    #[arg(long)]
    pub workflow: Option<String>,

    #[arg(long)]
    pub run: Option<String>,
}
