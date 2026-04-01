use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Initialize or migrate the fleet SQLite database")]
pub struct DbInitCommand;
