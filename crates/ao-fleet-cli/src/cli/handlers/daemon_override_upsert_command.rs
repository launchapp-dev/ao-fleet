use clap::Args;

#[derive(Debug, Args)]
pub struct DaemonOverrideUpsertCommand {
    #[arg(long)]
    pub team_id: String,

    #[arg(long)]
    pub mode: String,

    #[arg(long)]
    pub forced_state: Option<String>,

    #[arg(long)]
    pub pause_until: Option<String>,

    #[arg(long)]
    pub note: Option<String>,

    #[arg(long, default_value = "founder")]
    pub source: String,
}
