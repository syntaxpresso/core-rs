pub mod get_all_files_command;
pub mod services;

use std::path::PathBuf;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    GetAllFiles {
        #[arg(long)]
        cwd: PathBuf,
    },
}

impl Commands {
    pub fn execute(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Commands::GetAllFiles { cwd } => get_all_files_command::execute(cwd.clone()),
        }
    }
}
