use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Record a knowledge fact in the fleet knowledge base")]
pub struct KnowledgeFactCreateCommand {
    #[arg(long)]
    pub id: Option<String>,

    #[arg(long)]
    pub scope: String,

    #[arg(long)]
    pub scope_ref: Option<String>,

    #[arg(long)]
    pub kind: String,

    #[arg(long)]
    pub statement: String,

    #[arg(long)]
    pub confidence: u8,

    #[arg(long)]
    pub source_id: Option<String>,

    #[arg(long)]
    pub source_kind: Option<String>,

    #[arg(long = "tag")]
    pub tags: Vec<String>,

    #[arg(long)]
    pub observed_at: Option<String>,
}
