use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Update fields on a registered host")]
pub struct HostUpdateCommand {
    #[arg(long)]
    pub id: String,

    #[arg(long)]
    pub slug: Option<String>,

    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub address: Option<String>,

    #[arg(long)]
    pub platform: Option<String>,

    #[arg(long)]
    pub status: Option<String>,

    #[arg(long)]
    pub capacity_slots: Option<i32>,
}
