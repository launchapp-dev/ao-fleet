use clap::Args;

#[derive(Debug, Args)]
pub struct ConfigSnapshotExportCommand {
    #[arg(long)]
    pub output: Option<String>,
}
