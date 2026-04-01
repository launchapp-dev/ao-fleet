use clap::Args;

#[derive(Debug, Args)]
#[command(
    about = "Show live daemon status for a specific project",
    long_about = "Shows the project record together with its current desired and observed daemon \
state. Use --refresh to query the daemon live before displaying."
)]
pub struct ProjectStatusCommand {
    #[arg(long, help = "Project ID")]
    pub id: String,

    #[arg(long, default_value_t = false, help = "Query the daemon live before displaying")]
    pub refresh: bool,
}
