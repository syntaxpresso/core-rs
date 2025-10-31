pub mod create_java_file_command;
pub mod create_jpa_entity_basic_field_command;
pub mod create_jpa_entity_command;
pub mod create_jpa_repository_command;
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
        basic_field_config::BasicFieldConfig, java_field_temporal::JavaFieldTemporal,
        java_field_time_zone_storage::JavaFieldTimeZoneStorage, java_file_type::JavaFileType,
        java_source_directory_type::JavaSourceDirectoryType,
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
    CreateJPARepository {
        #[arg(long, value_parser = validate_directory, required = true)]
        cwd: PathBuf,

        #[arg(long, required = true)]
        entity_file_path: PathBuf,

        #[arg(long, required = false)]
        b64_superclass_source: Option<String>,
    },
    GetJPAEntityInfo {
        #[arg(long, value_parser = validate_directory, required = true)]
        cwd: PathBuf,

        #[arg(long, required = false)]
        entity_file_path: Option<PathBuf>,

        #[arg(long, required = false)]
        b64_source_code: Option<String>,
    },
    CreateJPAEntityBasicField {
        #[arg(long, value_parser = validate_directory, required = true)]
        cwd: PathBuf,

        #[arg(long, required = true)]
        entity_file_path: PathBuf,

        #[arg(long, required = true)]
        field_name: String,

        #[arg(long, required = true)]
        field_type: String,

        #[arg(long, required = false)]
        field_type_package_name: Option<String>,

        #[arg(long, required = false)]
        field_length: Option<u16>,

        #[arg(long, required = false)]
        field_precision: Option<u16>,

        #[arg(long, required = false)]
        field_scale: Option<u16>,

        #[arg(long, required = false)]
        field_temporal: Option<JavaFieldTemporal>,

        #[arg(long, required = false)]
        field_timezone_storage: Option<JavaFieldTimeZoneStorage>,

        #[arg(long)]
        field_unique: bool,

        #[arg(long)]
        field_nullable: bool,

        #[arg(long)]
        field_large_object: bool,
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
            Commands::CreateJPAEntity { cwd, package_name, file_name } => {
                let response =
                    create_jpa_entity_command::execute(cwd.as_path(), package_name, file_name);
                response.to_json_pretty().map_err(|e| e.into())
            }
            Commands::CreateJPARepository { cwd, entity_file_path, b64_superclass_source } => {
                let response = create_jpa_repository_command::execute(
                    cwd.as_path(),
                    entity_file_path.as_path(),
                    b64_superclass_source.as_deref(),
                );
                response.to_json_pretty().map_err(|e| e.into())
            }
            Commands::GetJPAEntityInfo { cwd, entity_file_path, b64_source_code } => {
                let response = get_jpa_entity_info_command::execute(
                    cwd.as_path(),
                    entity_file_path.as_deref(),
                    b64_source_code.as_deref(),
                );
                response.to_json_pretty().map_err(|e| e.into())
            }
            Commands::CreateJPAEntityBasicField {
                cwd,
                entity_file_path,
                field_name,
                field_type,
                field_type_package_name,
                field_length,
                field_precision,
                field_scale,
                field_temporal,
                field_timezone_storage,
                field_unique,
                field_nullable,
                field_large_object,
            } => {
                let field_config = BasicFieldConfig {
                    field_name: field_name.clone(),
                    field_type: field_type.clone(),
                    field_type_package_name: field_type_package_name.clone(),
                    field_length: *field_length,
                    field_precision: *field_precision,
                    field_scale: *field_scale,
                    field_temporal: field_temporal.clone(),
                    field_timezone_storage: field_timezone_storage.clone(),
                    field_unique: *field_unique,
                    field_nullable: *field_nullable,
                    field_large_object: *field_large_object,
                };
                let response = create_jpa_entity_basic_field_command::execute(
                    cwd.as_path(),
                    entity_file_path.as_path(),
                    &field_config,
                );
                response.to_json_pretty().map_err(|e| e.into())
            }
        }
    }
}
