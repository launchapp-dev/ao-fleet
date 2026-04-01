use clap::Args;

#[derive(Debug, Clone, Args)]
#[command(about = "Run an ao-cli JSON command against a registered project")]
pub struct ProjectAoJsonCommand {
    #[arg(long)]
    pub project_root: String,

    #[arg(trailing_var_arg = true, required = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}
