pub mod get_all_files_command;
pub mod services;
mod validators;

use std::path::PathBuf;

use clap::Subcommand;

use crate::commands::validators::directory_validator::validate_directory;

#[derive(Subcommand)]
pub enum Commands {
    GetAllFiles {
        #[arg(long, value_parser = validate_directory)]
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
