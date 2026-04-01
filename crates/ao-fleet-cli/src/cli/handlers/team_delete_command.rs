use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Delete a team from the fleet")]
pub struct TeamDeleteCommand {
    #[arg(long)]
    pub id: String,
}
