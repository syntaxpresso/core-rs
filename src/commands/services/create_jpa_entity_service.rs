use std::path::Path;

use crate::commands::services::create_java_file_service;
use crate::common::services::annotation_service;
use crate::common::ts_file::TSFile;
use crate::common::types::annotation_types::AnnotationInsertionPosition;
use crate::common::types::java_file_type::JavaFileType;
use crate::common::types::java_source_directory_type::JavaSourceDirectoryType;
use crate::responses::file_response::FileResponse;

fn build_file_response(ts_file: &TSFile, package_name: &str) -> Result<FileResponse, String> {
    let file_type_str = ts_file
        .get_file_name_without_ext()
        .ok_or("Failed to get file type string")?;
    let file_path = ts_file
        .file_path()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or("Failed to get file path")?;
    let file_package_name = package_name.to_string();
    Ok(FileResponse {
        file_type: file_type_str,
        file_path,
        file_package_name,
    })
}

fn create_java_file_and_get_response(
    cwd: &Path,
    package_name: &str,
    file_name: &str,
) -> Result<FileResponse, String> {
    let file_type = JavaFileType::Class;
    let source_directory = JavaSourceDirectoryType::Main;
    create_java_file_service::run(cwd, package_name, file_name, &file_type, &source_directory)
}

fn load_ts_file(file_path: &str) -> Result<TSFile, String> {
    TSFile::from_file(std::path::Path::new(file_path))
        .map_err(|e| format!("Failed to create TSFile: {}", e))
}

fn get_class_byte_position(ts_file: &TSFile) -> Result<usize, String> {
    let node = crate::common::services::class_declaration_service::get_public_class_node(ts_file);
    match node {
        Some(n) => Ok(n.start_byte()),
        None => Err("No public class found in file".to_string()),
    }
}

fn add_entity_annotation(ts_file: &mut TSFile, class_byte_position: usize) -> Result<(), String> {
    let position = AnnotationInsertionPosition::AboveScopeDeclaration;
    let result =
        annotation_service::add_annotation(ts_file, class_byte_position, &position, "@Entity");
    if result.is_none() {
        Err("Failed to add @Entity annotation".to_string())
    } else {
        Ok(())
    }
}

fn add_table_annotation(ts_file: &mut TSFile, class_byte_position: usize) -> Result<(), String> {
    let position = AnnotationInsertionPosition::AboveScopeDeclaration;
    let result =
        annotation_service::add_annotation(ts_file, class_byte_position, &position, "@Table");
    if result.is_none() {
        Err("Failed to add @Table annotation".to_string())
    } else {
        Ok(())
    }
}

fn add_table_name_argument(ts_file: &mut TSFile) -> Result<(), String> {
    let class_node =
        crate::common::services::class_declaration_service::get_public_class_node(ts_file)
            .ok_or("No public class found in file".to_string())?;
    let table_node = annotation_service::find_annotation_node_by_name(ts_file, class_node, "Table")
        .ok_or("@Table annotation not found".to_string())?;
    let table_byte_position = table_node.start_byte();
    let result = annotation_service::add_annotation_argument(
        ts_file,
        table_byte_position,
        "name",
        "\"test\"",
    );
    if result.is_none() {
        Err("Failed to add argument to @Table annotation".to_string())
    } else {
        Ok(())
    }
}

fn save_ts_file(ts_file: &mut TSFile, file_path: &str) -> Result<(), String> {
    let save_path = std::path::Path::new(file_path);
    ts_file
        .save_as(save_path)
        .map_err(|_| "Failed to save file".to_string())
}

pub fn run(cwd: &Path, package_name: &str, file_name: &str) -> Result<FileResponse, String> {
    // Step 1: Create the Java file and get the initial response
    let file_response = create_java_file_and_get_response(cwd, package_name, file_name)?;
    // Step 2: Load the file as a TSFile
    let mut ts_file = load_ts_file(&file_response.file_path)?;
    // Step 3: Get the public class node byte position
    let class_byte_position = get_class_byte_position(&ts_file)?;
    // Step 4: Add @Entity annotation above the class declaration
    add_entity_annotation(&mut ts_file, class_byte_position)?;
    // Step 5: Get the updated class node position after annotation insertion
    let updated_class_position = get_class_byte_position(&ts_file)?;
    // Step 6: Add @Table annotation above the class declaration
    add_table_annotation(&mut ts_file, updated_class_position)?;
    // Step 7: Add argument name = "test" to @Table annotation
    add_table_name_argument(&mut ts_file)?;
    // Step 8: Save the updated TSFile to disk
    save_ts_file(&mut ts_file, &file_response.file_path)?;
    // Step 9: Build and return the final file response
    build_file_response(&ts_file, package_name)
}
