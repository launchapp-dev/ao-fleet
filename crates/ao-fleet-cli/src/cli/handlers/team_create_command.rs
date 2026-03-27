use clap::Args;

#[derive(Debug, Args)]
pub struct TeamCreateCommand {
    #[arg(long)]
    pub slug: String,

    #[arg(long)]
    pub name: String,

    #[arg(long)]
    pub mission: String,

    #[arg(long)]
    pub ownership: String,

    #[arg(long, default_value_t = 0)]
    pub business_priority: i32,
}
