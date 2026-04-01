use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List recent audit log entries")]
pub struct AuditListCommand {
    #[arg(long)]
    pub team_id: Option<String>,

    #[arg(long)]
    pub limit: Option<usize>,
}
