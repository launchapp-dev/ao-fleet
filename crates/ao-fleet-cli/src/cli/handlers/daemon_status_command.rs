use clap::Args;

#[derive(Debug, Args)]
pub struct DaemonStatusCommand {
    #[arg(long)]
    pub team_id: Option<String>,

    #[arg(long, default_value_t = false)]
    pub refresh: bool,
}
