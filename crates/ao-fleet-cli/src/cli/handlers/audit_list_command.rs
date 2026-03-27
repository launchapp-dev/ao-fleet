use clap::Args;

#[derive(Debug, Args)]
pub struct AuditListCommand {
    #[arg(long)]
    pub team_id: Option<String>,

    #[arg(long)]
    pub limit: Option<usize>,
}
