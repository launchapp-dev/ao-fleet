use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Preview or apply daemon reconciliation actions across the fleet")]
pub struct DaemonReconcileCommand {
    #[arg(long)]
    pub at: Option<String>,

    #[arg(long)]
    pub team_id: Option<String>,

    #[arg(long, value_name = "team_id=count")]
    pub backlog: Vec<String>,

    #[arg(long, default_value_t = false)]
    pub apply: bool,
}
