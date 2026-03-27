use clap::Args;

#[derive(Debug, Args)]
pub struct KnowledgeSearchCommand {
    #[arg(long)]
    pub scope: Option<String>,

    #[arg(long)]
    pub scope_ref: Option<String>,

    #[arg(long = "document-kind")]
    pub document_kinds: Vec<String>,

    #[arg(long = "fact-kind")]
    pub fact_kinds: Vec<String>,

    #[arg(long = "source-kind")]
    pub source_kinds: Vec<String>,

    #[arg(long = "tag")]
    pub tags: Vec<String>,

    #[arg(long)]
    pub text: Option<String>,

    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}
