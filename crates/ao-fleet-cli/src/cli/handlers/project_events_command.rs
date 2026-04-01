use clap::Args;

#[derive(Debug, Clone, Args)]
#[command(about = "Stream or tail workflow events for a project")]
pub struct ProjectEventsCommand {
    #[arg(long)]
    pub project_root: String,

    #[arg(long)]
    pub workflow: Option<String>,

    #[arg(long)]
    pub run: Option<String>,

    #[arg(long)]
    pub cat: Option<String>,

    #[arg(long)]
    pub level: Option<String>,

    #[arg(long, default_value_t = 400)]
    pub tail: usize,

    #[arg(long)]
    pub follow: bool,
}
