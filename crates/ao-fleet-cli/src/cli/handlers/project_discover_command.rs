use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Discover AO projects under one or more directory trees")]
pub struct ProjectDiscoverCommand {
    #[arg(long = "search-root")]
    pub search_roots: Vec<String>,

    #[arg(long, default_value_t = 6)]
    pub max_depth: usize,

    #[arg(long, default_value_t = false)]
    pub register: bool,

    #[arg(long, default_value_t = false)]
    pub include_ao_shells: bool,

    #[arg(long)]
    pub team_id: Option<String>,
}
