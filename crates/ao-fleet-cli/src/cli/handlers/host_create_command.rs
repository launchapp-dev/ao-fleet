use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Register a new execution host in the fleet")]
pub struct HostCreateCommand {
    #[arg(long)]
    pub slug: String,

    #[arg(long)]
    pub name: String,

    #[arg(long)]
    pub address: String,

    #[arg(long)]
    pub platform: String,

    #[arg(long, default_value = "healthy")]
    pub status: String,

    #[arg(long, default_value_t = 1)]
    pub capacity_slots: i32,
}
