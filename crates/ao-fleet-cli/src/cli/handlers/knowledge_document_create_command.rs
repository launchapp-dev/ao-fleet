use clap::Args;

#[derive(Debug, Args)]
pub struct KnowledgeDocumentCreateCommand {
    #[arg(long)]
    pub id: Option<String>,

    #[arg(long)]
    pub scope: String,

    #[arg(long)]
    pub scope_ref: Option<String>,

    #[arg(long)]
    pub kind: String,

    #[arg(long)]
    pub title: String,

    #[arg(long)]
    pub summary: String,

    #[arg(long)]
    pub body: String,

    #[arg(long)]
    pub source_id: Option<String>,

    #[arg(long)]
    pub source_kind: Option<String>,

    #[arg(long = "tag")]
    pub tags: Vec<String>,
}
