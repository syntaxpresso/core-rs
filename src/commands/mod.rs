pub mod create_java_file_command;
pub mod create_jpa_entity_command;
// pub mod create_jpa_repository_command;
pub mod get_all_files_command;
pub mod get_jpa_entity_info_command;
pub mod services;
mod validators;

use std::path::PathBuf;

use clap::Subcommand;

use crate::{
    commands::validators::{
        directory_validator::validate_directory,
        java_class_name_validator::validate_java_class_name,
        package_name_validator::validate_package_name,
    },
    common::types::{
        java_file_type::JavaFileType, java_source_directory_type::JavaSourceDirectoryType,
    },
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

        #[arg(long, required = true)]
        file_type: JavaFileType,

        #[arg(long, default_value = "main")]
        source_directory: JavaSourceDirectoryType,
    },
    CreateJPAEntity {
        #[arg(long, value_parser = validate_directory, required = true)]
        cwd: PathBuf,

        #[arg(long, value_parser = validate_package_name, required = true)]
        package_name: String,

        #[arg(long, value_parser = validate_java_class_name, required = true)]
        file_name: String,
    },
    // CreateJPARepository {
    //     #[arg(long, value_parser = validate_directory, required = true)]
    //     cwd: PathBuf,
    //
    //     #[arg(long, required = true)]
    //     entity_file_path: PathBuf,
    //
    //     #[arg(long, required = false)]
    //     b64_superclass_source: Option<String>,
    // },
    GetJPAEntityInfo {
        #[arg(long, value_parser = validate_directory, required = true)]
        cwd: PathBuf,

        #[arg(long, required = false)]
        entity_file_path: Option<PathBuf>,

        #[arg(long, required = false)]
        b64_source_code: Option<String>,
    },
}

impl Commands {
    pub fn execute(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Commands::GetAllFiles { cwd } => {
                let response = get_all_files_command::execute(cwd.as_path());
                response.to_json_pretty().map_err(|e| e.into())
            }
            Commands::CreateJavaFile {
                cwd,
                package_name,
                file_name,
                file_type,
                source_directory,
            } => {
                let response = create_java_file_command::execute(
                    cwd.as_path(),
                    package_name,
                    file_name,
                    file_type,
                    source_directory,
                );
                response.to_json_pretty().map_err(|e| e.into())
            }
            Commands::CreateJPAEntity {
                cwd,
                package_name,
                file_name,
            } => {
                let response =
                    create_jpa_entity_command::execute(cwd.as_path(), package_name, file_name);
                response.to_json_pretty().map_err(|e| e.into())
            }
            // Commands::CreateJPARepository {
            //     cwd,
            //     entity_file_path,
            //     b64_superclass_source,
            // } => {
            //     let response = create_jpa_repository_command::execute(
            //         cwd.as_path(),
            //         entity_file_path.as_path(),
            //         b64_superclass_source.as_deref(),
            //     );
            //     response.to_json_pretty().map_err(|e| e.into())
            // }
            Commands::GetJPAEntityInfo {
                cwd,
                entity_file_path,
                b64_source_code,
            } => {
                let response = get_jpa_entity_info_command::execute(
                    cwd.as_path(),
                    entity_file_path.as_deref(),
                    b64_source_code.as_deref(),
                );
                response.to_json_pretty().map_err(|e| e.into())
            }
        }
    }
}
