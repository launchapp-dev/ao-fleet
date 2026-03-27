use clap::Args;

#[derive(Debug, Args)]
pub struct ProjectGetCommand {
    #[arg(long)]
    pub id: String,
}
