use clap::Args;

#[derive(Debug, Args)]
pub struct TeamDeleteCommand {
    #[arg(long)]
    pub id: String,
}
