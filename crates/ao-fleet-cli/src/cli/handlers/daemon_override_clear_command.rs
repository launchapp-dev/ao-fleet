use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Clear a daemon schedule override for a team")]
pub struct DaemonOverrideClearCommand {
    #[arg(long)]
    pub team_id: String,
}
