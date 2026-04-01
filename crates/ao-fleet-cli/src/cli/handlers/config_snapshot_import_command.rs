use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Import fleet config from a snapshot file")]
pub struct ConfigSnapshotImportCommand {
    #[arg(long)]
    pub input: String,
}
