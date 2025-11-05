use std::path::Path;

use crate::commands::services::create_java_file_service::{
  build_save_path, correct_java_file_name, create_ts_file, generate_file_template,
};
use crate::common::services::annotation_service;
use crate::common::services::class_declaration_service::{
  get_class_declaration_name_node, get_public_class_node,
};
use crate::common::services::import_declaration_service::{self, add_import};
use crate::common::ts_file::TSFile;
use crate::common::types::annotation_types::AnnotationInsertionPosition;
use crate::common::types::import_types::ImportInsertionPosition;
use crate::common::types::java_file_type::JavaFileType;
use crate::common::types::java_source_directory_type::JavaSourceDirectoryType;
use crate::common::utils::case_util;
use crate::responses::file_response::FileResponse;

fn add_jpa_imports(ts_file: &mut TSFile) -> Result<(), String> {
  let entity_import_result = import_declaration_service::add_import(
    ts_file,
    &ImportInsertionPosition::AfterLastImport,
    "jakarta.persistence",
    "Entity",
  );
  if entity_import_result.is_none() {
    return Err("Failed to add import for jakarta.persistence.Entity".to_string());
  }
  let table_import_result = import_declaration_service::add_import(
    ts_file,
    &ImportInsertionPosition::AfterLastImport,
    "jakarta.persistence",
    "Table",
  );
  if table_import_result.is_none() {
    return Err("Failed to add import for jakarta.persistence.Table".to_string());
  }
  Ok(())
}

fn build_file_response(ts_file: &TSFile, package_name: &str) -> Result<FileResponse, String> {
  let file_type_str =
    ts_file.get_file_name_without_ext().ok_or("Failed to get file type string")?;
  let file_path = ts_file
    .file_path()
    .map(|p| p.to_string_lossy().to_string())
    .ok_or("Failed to get file path")?;
  let file_package_name = package_name.to_string();
  Ok(FileResponse { file_type: file_type_str, file_path, file_package_name })
}

fn create_java_file_and_get_response(
  package_name: &str,
  file_name: &str,
) -> Result<TSFile, String> {
  let file_type = JavaFileType::Class;
  let file_template = generate_file_template(&file_type, package_name, file_name);
  let ts_file = create_ts_file(&file_template);
  Ok(ts_file)
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
  if result.is_none() { Err("Failed to add @Entity annotation".to_string()) } else { Ok(()) }
}

fn add_table_annotation(ts_file: &mut TSFile, class_byte_position: usize) -> Result<(), String> {
  let position = AnnotationInsertionPosition::AboveScopeDeclaration;
  let result =
    annotation_service::add_annotation(ts_file, class_byte_position, &position, "@Table");
  if result.is_none() { Err("Failed to add @Table annotation".to_string()) } else { Ok(()) }
}

fn add_table_name_argument(ts_file: &mut TSFile, class_name: &str) -> Result<(), String> {
  let class_node =
    crate::common::services::class_declaration_service::get_public_class_node(ts_file)
      .ok_or("No public class found in file".to_string())?;
  let table_node = annotation_service::find_annotation_node_by_name(ts_file, class_node, "Table")
    .ok_or("@Table annotation not found".to_string())?;
  let table_byte_position = table_node.start_byte();
  let table_name = case_util::to_snake_case(class_name);
  let table_name_value = format!("\"{}\"", table_name);
  let result = annotation_service::add_annotation_argument(
    ts_file,
    table_byte_position,
    "name",
    &table_name_value,
  );
  if result.is_none() {
    Err("Failed to add argument to @Table annotation".to_string())
  } else {
    Ok(())
  }
}

fn add_superclass_heritage(
  ts_file: &mut TSFile,
  superclass_type_opt: Option<&str>,
  superclass_package_name_opt: Option<&str>,
) -> Result<(), String> {
  if superclass_type_opt.is_some() != superclass_package_name_opt.is_some() {
    return Err("Both superclass type and it's package name are necessary".to_string());
  }
  if superclass_type_opt.is_some() && superclass_package_name_opt.is_some() {
    let superclass_type = superclass_type_opt.ok_or("Superclass type not provided".to_string())?;
    let superclass_package_name =
      superclass_package_name_opt.ok_or("Superclass package name not provided".to_string())?;
    let class_declaration_node = get_public_class_node(ts_file)
      .ok_or("Unable to get public class declaration from JPA Entity".to_string())?;
    let class_name_node = get_class_declaration_name_node(ts_file, class_declaration_node)
      .ok_or("Unable to get public class name node from JPA Entity".to_string())?;
    ts_file.insert_text(class_name_node.end_byte(), &format!(" extends {}", superclass_type));
    add_import(
      ts_file,
      &ImportInsertionPosition::AfterLastImport,
      superclass_package_name,
      superclass_type,
    );
  }
  Ok(())
}

fn save_ts_file(
  ts_file: &mut TSFile,
  cwd: &Path,
  file_name: &str,
  package_name: &str,
) -> Result<(), String> {
  let corrected_file_name = correct_java_file_name(file_name);
  let save_path =
    build_save_path(&JavaSourceDirectoryType::Main, cwd, package_name, &corrected_file_name)?;
  ts_file.save_as(&save_path, cwd).map_err(|e| format!("Failed to save file: {}", e))
}

pub fn run(
  cwd: &Path,
  package_name: &str,
  file_name: &str,
  superclass_type: Option<&str>,
  superclass_package_name: Option<&str>,
) -> Result<FileResponse, String> {
  // Normalize the class name to PascalCase
  let normalized_class_name = case_util::to_pascal_case(file_name);
  // Step 1: Create the Java file
  let mut ts_file = create_java_file_and_get_response(package_name, &normalized_class_name)?;
  // Step 2: Add required imports for JPA annotations
  add_jpa_imports(&mut ts_file)?;
  // Step 3: Get the public class node byte position after imports are added
  let class_byte_position = get_class_byte_position(&ts_file)?;
  // Step 4: Add @Entity annotation above the class declaration
  add_entity_annotation(&mut ts_file, class_byte_position)?;
  // Step 5: Get the updated class node position after annotation insertion
  let updated_class_position = get_class_byte_position(&ts_file)?;
  // Step 6: Add @Table annotation above the class declaration
  add_table_annotation(&mut ts_file, updated_class_position)?;
  // Step 7: Add table name argument with snake_case conversion
  add_table_name_argument(&mut ts_file, &normalized_class_name)?;
  // Step 8: Add superclass heritage
  add_superclass_heritage(&mut ts_file, superclass_type, superclass_package_name)?;
  // Step 9: Save the updated TSFile to disk
  save_ts_file(&mut ts_file, cwd, file_name, package_name)?;
  // Step 10: Build and return the final file response
  build_file_response(&ts_file, package_name)
}
