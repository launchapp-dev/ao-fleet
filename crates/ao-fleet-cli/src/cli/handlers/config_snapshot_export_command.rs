use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Export fleet config to a snapshot file")]
pub struct ConfigSnapshotExportCommand {
    #[arg(long)]
    pub output: Option<String>,
}
