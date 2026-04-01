use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List knowledge sources, optionally filtered by scope")]
pub struct KnowledgeSourceListCommand {
    #[arg(long)]
    pub scope: Option<String>,

    #[arg(long)]
    pub scope_ref: Option<String>,

    #[arg(long, default_value_t = 100)]
    pub limit: usize,
}
