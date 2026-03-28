use clap::Args;

#[derive(Debug, Args)]
pub struct DaemonOverrideClearCommand {
    #[arg(long)]
    pub team_id: String,
}
