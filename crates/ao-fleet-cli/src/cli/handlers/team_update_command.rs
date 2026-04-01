use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Update fields on a registered team")]
pub struct TeamUpdateCommand {
    #[arg(long)]
    pub id: String,

    #[arg(long)]
    pub slug: Option<String>,

    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub mission: Option<String>,

    #[arg(long)]
    pub ownership: Option<String>,

    #[arg(long)]
    pub business_priority: Option<i32>,
}
