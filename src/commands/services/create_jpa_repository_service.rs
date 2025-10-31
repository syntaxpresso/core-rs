use std::path::{Path, PathBuf};

use crate::commands::services::create_java_file_service::{self};
use crate::commands::services::get_jpa_entity_info_service::{self};
use crate::common::services::import_declaration_service::add_import;
use crate::common::services::interface_declaration_service::{
    get_interface_name_node, get_public_interface_node,
};
use crate::common::services::package_declaration_service::{
    get_package_declaration_node, get_package_scope_node,
};
use crate::common::ts_file::TSFile;
use crate::common::types::import_types::ImportInsertionPosition;
use crate::common::types::java_file_type::JavaFileType;
use crate::common::types::java_source_directory_type::JavaSourceDirectoryType;
use crate::responses::create_jpa_repository_response::CreateJPARepositoryResponse;
use crate::responses::file_response::FileResponse;
use crate::responses::get_jpa_entity_info_response::GetJpaEntityInfoResponse;

fn create_repository_file(cwd: &Path, entity_ts_file: &TSFile) -> Result<FileResponse, String> {
    let entity_package_declaration_node = get_package_declaration_node(entity_ts_file);
    if entity_package_declaration_node.is_none() {
        return Err("Unable to get JPA Entity package declaration node".to_string());
    }
    let entity_package_scope_node =
        get_package_scope_node(entity_ts_file, entity_package_declaration_node.unwrap());
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

pub fn create_jpa_repository_response(
    id_field_found: bool,
    superclass_type: Option<String>,
    file_response: Option<FileResponse>,
) -> CreateJPARepositoryResponse {
    CreateJPARepositoryResponse { id_field_found, superclass_type, repository: file_response }
}

fn create_file_response(ts_file: &TSFile) -> Result<FileResponse, String> {
    let file_path = ts_file
        .file_path()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .ok_or_else(|| "Unable to get file path from TSFile".to_string())?;
    let file_type = ts_file.get_file_name_without_ext().unwrap_or_default();
    let package_declaration_node =
        crate::common::services::package_declaration_service::get_package_declaration_node(ts_file);
    let package_name = package_declaration_node
        .and_then(|node| {
            crate::common::services::package_declaration_service::get_package_scope_node(
                ts_file, node,
            )
        })
        .and_then(|scope_node| ts_file.get_text_from_node(&scope_node).map(|s| s.to_string()))
        .unwrap_or_default();
    Ok(FileResponse { file_path, file_type, file_package_name: package_name })
}

fn create_and_extend_jpa_repository(
    cwd: &Path,
    entity_ts_file: &TSFile,
    entity_type: &str,
    jpa_entity_info: &GetJpaEntityInfoResponse,
) -> Result<TSFile, String> {
    let create_repository_file_response = create_repository_file(cwd, entity_ts_file)?;
    let jpa_repository_path = PathBuf::from(&create_repository_file_response.file_path);
    let mut jpa_repository_ts_file = TSFile::from_file(jpa_repository_path.as_path())
        .map_err(|e| format!("Unable to parse newly created repository file: {}", e))?;
    extend_jpa_repository(
        &mut jpa_repository_ts_file,
        entity_type,
        jpa_entity_info.id_field_type.as_ref().unwrap().as_ref(),
        jpa_entity_info.id_field_package_name.as_ref().unwrap().as_ref(),
    );
    Ok(jpa_repository_ts_file)
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
    let jpa_repository_ext_str =
        format!(" extends JpaRepository<{}, {}> ", entity_type, id_field_type);
    jpa_repository_ts_file
        .insert_text(public_interface_name_node.unwrap().end_byte(), &jpa_repository_ext_str);
    let package_declaration_node = get_package_declaration_node(jpa_repository_ts_file);
    if package_declaration_node.is_none() {
        return;
    }
    let import_insert_position_one = ImportInsertionPosition::AfterPackageDeclaration;
    let import_insert_position_two = ImportInsertionPosition::AfterLastImport;
    add_import(
        jpa_repository_ts_file,
        &import_insert_position_one,
        id_field_package_name,
        id_field_type,
    );
    add_import(
        jpa_repository_ts_file,
        &import_insert_position_two,
        "org.springframework.data.jpa.repository",
        "JpaRepository",
    );
}

fn step_parse_entity_file(entity_file_path: &Path) -> Result<TSFile, String> {
    TSFile::from_file(entity_file_path).map_err(|_| "Unable to parse JPA Entity file".to_string())
}

fn step_extract_entity_type(entity_ts_file: &TSFile) -> String {
    entity_ts_file.get_file_name_without_ext().unwrap_or_else(|| "Unknown".to_string())
}

fn step_get_jpa_entity_info(
    entity_file_path: Option<&Path>,
    b64_source_code: Option<&str>,
) -> Result<GetJpaEntityInfoResponse, String> {
    get_jpa_entity_info_service::run(entity_file_path, b64_source_code)
}

fn step_process_entity_without_superclass(
    cwd: &Path,
    entity_ts_file: &TSFile,
    entity_type: &str,
    jpa_entity_info: &GetJpaEntityInfoResponse,
) -> Result<CreateJPARepositoryResponse, String> {
    if jpa_entity_info.is_jpa_entity
        && jpa_entity_info.id_field_type.is_some()
        && jpa_entity_info.id_field_package_name.is_some()
    {
        step_create_repository_and_save(cwd, entity_ts_file, entity_type, jpa_entity_info)
    } else {
        let superclass_type = jpa_entity_info.superclass_type.clone();
        let response = create_jpa_repository_response(false, superclass_type, None);
        Ok(response)
    }
}

fn step_process_entity_with_superclass(
    cwd: &Path,
    entity_ts_file: &TSFile,
    entity_type: &str,
    jpa_entity_info: &GetJpaEntityInfoResponse,
) -> Result<CreateJPARepositoryResponse, String> {
    // If the superclass is not a JPA entity but has a superclass (like MappedSuperclass extending BaseEntity),
    // return the superclass type instead of trying to create a repository
    if !jpa_entity_info.is_jpa_entity && jpa_entity_info.superclass_type.is_some() {
        let superclass_type = jpa_entity_info.superclass_type.clone();
        let response = create_jpa_repository_response(false, superclass_type, None);
        return Ok(response);
    }
    if jpa_entity_info.id_field_type.is_none() || jpa_entity_info.id_field_package_name.is_none() {
        return Err("Unable to find ID field for this JPA Entity".to_string());
    }
    step_create_repository_and_save(cwd, entity_ts_file, entity_type, jpa_entity_info)
}

fn step_create_repository_and_save(
    cwd: &Path,
    entity_ts_file: &TSFile,
    entity_type: &str,
    jpa_entity_info: &GetJpaEntityInfoResponse,
) -> Result<CreateJPARepositoryResponse, String> {
    let mut jpa_repository_ts_file =
        create_and_extend_jpa_repository(cwd, entity_ts_file, entity_type, jpa_entity_info)?;
    match jpa_repository_ts_file.save() {
        Ok(_) => {
            let file_response = create_file_response(&jpa_repository_ts_file)?;
            let response = create_jpa_repository_response(true, None, Some(file_response));
            Ok(response)
        }
        Err(_) => Err("Unable to create response".to_string()),
    }
}

pub fn run(
    cwd: &Path,
    entity_file_path: &Path,
    b64_superclass_source: Option<&str>,
) -> Result<CreateJPARepositoryResponse, String> {
    // Step 1: Parse JPA Entity file
    let entity_ts_file = step_parse_entity_file(entity_file_path)?;
    // Step 2: Extract entity type from file name
    let entity_type = step_extract_entity_type(&entity_ts_file);
    if b64_superclass_source.is_none() {
        // Step 3: Get JPA entity info from entity file
        let jpa_entity_info = step_get_jpa_entity_info(Some(entity_file_path), None)?;
        // Step 4: Process entity without superclass
        step_process_entity_without_superclass(cwd, &entity_ts_file, &entity_type, &jpa_entity_info)
    } else {
        // Step 3: Get JPA entity info from superclass source
        let jpa_entity_info = step_get_jpa_entity_info(None, b64_superclass_source)?;
        // Step 4: Process entity with superclass
        step_process_entity_with_superclass(cwd, &entity_ts_file, &entity_type, &jpa_entity_info)
    }
}
