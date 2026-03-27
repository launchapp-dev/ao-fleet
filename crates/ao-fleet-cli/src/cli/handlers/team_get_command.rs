use clap::Args;

#[derive(Debug, Args)]
pub struct TeamGetCommand {
    #[arg(long)]
    pub id: String,
}
