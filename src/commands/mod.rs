pub mod create_java_file_command;
pub mod get_all_files_command;
pub mod services;
mod validators;

use std::path::PathBuf;

use clap::Subcommand;

use crate::{
    commands::validators::{
        directory_validator::validate_directory,
        java_class_name_validator::validate_java_class_name,
        java_identifier_validator::validate_java_identifier,
        package_name_validator::validate_package_name,
    },
    common::types::java_file_type::JavaFileType,
};

#[derive(Subcommand)]
pub enum Commands {
    GetAllFiles {
        #[arg(long, value_parser = validate_directory, required = true)]
        cwd: PathBuf,
    },
    CreateJavaFile {
        #[arg(long, value_parser = validate_directory, required = true)]
        cwd: PathBuf,

        #[arg(long, value_parser = validate_package_name, required = true)]
        package_name: String,

        #[arg(long, value_parser = validate_java_class_name, required = true)]
        file_name: String,

        #[arg(long, value_parser = validate_java_identifier, required = true)]
        file_type: JavaFileType,
    },
}

impl Commands {
    pub fn execute(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Commands::GetAllFiles { cwd } => {
                let response = get_all_files_command::execute(cwd.clone());
                response.to_json_pretty().map_err(|e| e.into())
            }
            Commands::CreateJavaFile {
                cwd,
                package_name,
                file_name,
                file_type,
            } => {
                let response = get_all_files_command::execute(cwd.clone());
                response.to_json_pretty().map_err(|e| e.into())
            }
        }
    }
}
