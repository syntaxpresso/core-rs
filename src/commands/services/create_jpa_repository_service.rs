use std::path::{Path, PathBuf};

use crate::commands::services::create_java_file_service::{self};
use crate::commands::services::get_jpa_entity_info_service::{self};
use crate::common::services::import_declaration_service::add_import;
use crate::common::services::interface_declaration_service::{
    get_interface_name_node, get_public_interface_node,
};
use crate::common::services::package_declaration_service::{
    get_package_class_scope_node, get_package_declaration_node,
};
use crate::common::ts_file::TSFile;
use crate::common::types::import_types::ImportInsertionPosition;
use crate::common::types::java_file_type::JavaFileType;
use crate::common::types::java_source_directory_type::JavaSourceDirectoryType;
use crate::responses::file_response::FileResponse;
use crate::responses::get_jpa_entity_info_response::GetJpaEntityInfoResponse;

// fn prepare_repository_data(
//     entity_type: &str,
//     entity_package: &str,
//     id_field_type: &str,
//     id_field_package: &str,
// ) -> Result<RepositoryData, String> {
//     let repository_name = format!("{}Repository", entity_type);
//     let repository_package = entity_package.to_string();
//
//     Ok(RepositoryData {
//         repository_name,
//         repository_package,
//         entity_type: entity_type.to_string(),
//         entity_package: entity_package.to_string(),
//         id_field_type: id_field_type.to_string(),
//         id_field_package: id_field_package.to_string(),
//     })
// }
//
fn create_repository_file(cwd: &Path, entity_ts_file: &TSFile) -> Result<FileResponse, String> {
    let entity_package_declaration_node = get_package_declaration_node(entity_ts_file);
    if entity_package_declaration_node.is_none() {
        return Err("Unable to get JPA Entity package declaration node".to_string());
    }
    let entity_package_scope_node =
        get_package_class_scope_node(entity_ts_file, entity_package_declaration_node.unwrap());
    if entity_package_scope_node.is_none() {
        return Err("Unable to get JPA Entity package scope node".to_string());
    }
    let entity_package_name = entity_ts_file
        .get_text_from_node(&entity_package_scope_node.unwrap())
        .map(|package_name| package_name.to_string())
        .ok_or_else(|| "Unable to extract package name from package scope node".to_string())?;
    let repository_file_name = entity_ts_file
        .get_file_name_without_ext()
        .map(|file_name| format!("{}Repository", file_name))
        .ok_or_else(|| "Unable to get entity file name".to_string())?;
    let repository_file_type = JavaFileType::Interface;
    let repository_source_dir_type = JavaSourceDirectoryType::Main;
    let create_java_file_response = create_java_file_service::run(
        cwd,
        &entity_package_name,
        &repository_file_name,
        &repository_file_type,
        &repository_source_dir_type,
    )?;
    Ok(create_java_file_response)
}

fn create_and_extend_jpa_repository(
    cwd: &Path,
    entity_ts_file: &TSFile,
    entity_type: &str,
    jpa_entity_info: &GetJpaEntityInfoResponse,
) -> Result<(), String> {
    let create_repository_file_response = create_repository_file(cwd, entity_ts_file)?;
    let jpa_repository_path = PathBuf::from(&create_repository_file_response.file_path);
    let mut jpa_repository_ts_file =
        TSFile::from_file(jpa_repository_path.as_path()).map_err(|e| {
            format!(
                "Unable to parse newly created repository file: {}",
                e.to_string()
            )
        })?;
    extend_jpa_repository(
        &mut jpa_repository_ts_file,
        entity_type,
        jpa_entity_info.id_field_type.as_ref().unwrap().as_ref(),
        jpa_entity_info
            .id_field_package_name
            .as_ref()
            .unwrap()
            .as_ref(),
    );
    jpa_repository_ts_file.save();
    Ok(())
}

fn extend_jpa_repository(
    jpa_repository_ts_file: &mut TSFile,
    entity_type: &str,
    id_field_type: &str,
    id_field_package_name: &str,
) {
    let public_interface_node = get_public_interface_node(jpa_repository_ts_file);
    if public_interface_node.is_none() {
        return;
    }
    let public_interface_name_node =
        get_interface_name_node(jpa_repository_ts_file, public_interface_node.unwrap());
    let jpa_repository_ext_str = format!(
        " extends JpaRepository<{}, {}> ",
        entity_type, id_field_type
    );
    jpa_repository_ts_file.insert_text(
        public_interface_name_node.unwrap().end_byte(),
        &jpa_repository_ext_str,
    );
    let package_declaration_node = get_package_declaration_node(jpa_repository_ts_file);
    if package_declaration_node.is_none() {
        return;
    }
    let import_insert_position = ImportInsertionPosition::AfterPackageDeclaration;
    add_import(
        jpa_repository_ts_file,
        &import_insert_position,
        format!("{}.{}", id_field_package_name, id_field_type).as_ref(),
    );
}

fn parse_entity_file(path: &Path) -> Result<TSFile, String> {
    match TSFile::from_file(path) {
        Ok(ts_file) => Ok(ts_file),
        Err(_) => Err("Unable to parse JPA Entity file".to_string()),
    }
}

fn get_jpa_entity_info(
    entity_file_path: Option<&Path>,
    b64_source_code: Option<&str>,
) -> Result<GetJpaEntityInfoResponse, String> {
    get_jpa_entity_info_service::run(entity_file_path, b64_source_code)
}

pub fn run(
    cwd: &Path,
    entity_file_path: &Path,
    b64_superclass_source: Option<&str>,
) -> Result<FileResponse, String> {
    // Step 1: Parse JPA Entity file
    let entity_ts_file = parse_entity_file(entity_file_path)?;
    let entity_type = entity_ts_file
        .get_file_name_without_ext()
        .unwrap_or_else(|| "Unknown".to_string());
    if b64_superclass_source.is_none() {
        let jpa_entity_info = get_jpa_entity_info(Some(entity_file_path), None)?;
        if jpa_entity_info.is_jpa_entity
            && jpa_entity_info.id_field_type.is_some()
            && jpa_entity_info.id_field_package_name.is_some()
        {
            create_and_extend_jpa_repository(
                cwd,
                &entity_ts_file,
                entity_type.as_ref(),
                &jpa_entity_info,
            )?;
        }
    }
    Ok(FileResponse {
        file_path: "test.java".to_string(),
        file_type: "test.java".to_string(),
        file_package_name: "test.java".to_string(),
    })

    // let entity_content = fs::read_to_string(entity_file_path)
    //     .map_err(|e| format!("Failed to read entity file: {}", e))?;
    //
    // let b64_entity_content = base64::prelude::BASE64_STANDARD.encode(entity_content.as_bytes());
    //
    // // Get entity information using our GetJpaEntityInfo service
    // // let entity_info = get_jpa_entity_info_service::run(
    //     &b64_entity_content,
    //     entity_file_path.to_str().unwrap_or(""),
    // )?;
    //
    // if !entity_info.is_jpa_entity {
    //     return Err("The provided file is not a JPA entity".to_string());
    // }
    //
    // // Handle missing ID field - check superclass if provided
    // let (id_field_type, id_field_package) = match (
    //     &entity_info.id_field_type,
    //     &entity_info.id_field_package_name,
    // ) {
    //     (Some(field_type), Some(field_package)) => (field_type.clone(), field_package.clone()),
    //     (None, None) => {
    //         // If no ID field found and we have superclass source, check it
    //         if let Some(b64_superclass) = b64_superclass_source {
    //             // let superclass_info = get_jpa_entity_info_service::run(b64_superclass, "")?;
    //             match (
    //                 &superclass_info.id_field_type,
    //                 &superclass_info.id_field_package_name,
    //             ) {
    //                 (Some(field_type), Some(field_package)) => {
    //                     (field_type.clone(), field_package.clone())
    //                 }
    //                 _ => return Err("No @Id field found in entity or its superclass".to_string()),
    //             }
    //         } else {
    //             return Err("No @Id field found in entity and no superclass provided".to_string());
    //         }
    //     }
    //     _ => return Err("Incomplete ID field information".to_string()),
    // };
    //
    // // Prepare repository data
    // let repository_data = prepare_repository_data(
    //     &entity_info.entity_type,
    //     &entity_info.entity_package_name,
    //     &id_field_type,
    //     &id_field_package,
    // )?;
    //
    // // Create repository file
    // let repository_file_response = create_repository_file(cwd, entity_file_path, repository_data)?;
    //
    // Ok(repository_file_response)
}
