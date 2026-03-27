use clap::Args;

#[derive(Debug, Args)]
pub struct ProjectDiscoverCommand {
    #[arg(long = "search-root")]
    pub search_roots: Vec<String>,

    #[arg(long, default_value_t = 6)]
    pub max_depth: usize,

    #[arg(long, default_value_t = false)]
    pub register: bool,

    #[arg(long)]
    pub team_id: Option<String>,
}
