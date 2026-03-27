use clap::Args;

#[derive(Debug, Args)]
pub struct ConfigSnapshotImportCommand {
    #[arg(long)]
    pub input: String,
}
