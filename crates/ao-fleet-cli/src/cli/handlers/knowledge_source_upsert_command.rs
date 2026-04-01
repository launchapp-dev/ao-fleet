use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Create or update a knowledge source")]
pub struct KnowledgeSourceUpsertCommand {
    #[arg(long)]
    pub id: Option<String>,

    #[arg(long)]
    pub kind: String,

    #[arg(long)]
    pub label: String,

    #[arg(long)]
    pub uri: Option<String>,

    #[arg(long)]
    pub scope: String,

    #[arg(long)]
    pub scope_ref: Option<String>,

    #[arg(long)]
    pub sync_state: String,

    #[arg(long)]
    pub last_synced_at: Option<String>,

    #[arg(long, default_value = "{}")]
    pub metadata_json: String,
}
