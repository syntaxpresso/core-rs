use crate::common::services::class_declaration_service::get_public_class_node;
use crate::common::services::field_declaration_service::{
  AddFieldDeclarationParams, add_field_declaration,
};
use crate::common::services::import_declaration_service::add_import;
use crate::common::services::package_declaration_service::{
  get_package_class_scope_node, get_package_declaration_node,
};
use crate::common::ts_file::TSFile;
use crate::common::types::enum_field_config::EnumFieldConfig;
use crate::common::types::import_types::ImportInsertionPosition;
use crate::common::types::java_basic_types::FieldInsertionPosition;
use crate::common::types::java_enum_type::JavaEnumType;
use crate::common::types::java_visibility_modifier::JavaVisibilityModifier;
use crate::common::utils::case_util::{self, CaseType};
use crate::responses::file_response::FileResponse;
use std::collections::HashMap;
use std::path::Path;

fn add_to_import_map(
  import_map: &mut HashMap<String, String>,
  package_name: &str,
  class_name: &str,
) {
  if !import_map.contains_key(class_name) {
    import_map.insert(class_name.to_string(), package_name.to_string());
  }
}

fn add_imports(ts_file: &mut TSFile, import_map: &HashMap<String, String>) {
  let import_position = ImportInsertionPosition::BeforeFirstImport;
  for (class_name, package_name) in import_map {
    add_import(ts_file, &import_position, package_name, class_name);
  }
}

fn add_field_and_annotations(
  ts_file: &mut TSFile,
  field_config: &EnumFieldConfig,
  import_map: &mut HashMap<String, String>,
) -> Result<(), String> {
  let column_name_snake_case =
    case_util::auto_convert_case(&field_config.field_name, CaseType::Snake);
  let public_class_node = get_public_class_node(ts_file)
    .ok_or_else(|| "Unable to get public class node from Entity".to_string())?;
  let public_class_node_start_byte = public_class_node.start_byte();
  // Add required imports based on field configuration
  add_to_import_map(import_map, "jakarta.persistence", "Column");
  add_to_import_map(import_map, "jakarta.persistence", "Enumerated");
  add_to_import_map(import_map, "jakarta.persistence", "EnumType");
  // Add enum type import
  add_to_import_map(import_map, &field_config.enum_package_name, &field_config.enum_type);
  let params = AddFieldDeclarationParams {
    insertion_position: FieldInsertionPosition::EndOfClassBody,
    visibility_modifier: JavaVisibilityModifier::Private,
    field_modifiers: vec![],
    field_type: &field_config.enum_type,
    field_name: &field_config.field_name,
    field_initialization: None,
  };
  add_field_declaration(ts_file, public_class_node_start_byte, params, |builder| {
    // Add @Enumerated annotation
    builder.add_annotation("@Enumerated")?.with_argument(
      "@Enumerated",
      "value",
      &format!("EnumType.{}", field_config.enum_type_storage.as_str()),
    )?;
    // Add @Column annotation
    builder.add_annotation("@Column")?.with_argument(
      "@Column",
      "name",
      &format!("\"{}\"", &column_name_snake_case),
    )?;
    // Add length for STRING type if specified and different from default (255)
    if matches!(field_config.enum_type_storage, JavaEnumType::String)
      && let Some(length) = field_config.field_length
      && length != 255
    {
      builder.with_argument("@Column", "length", &length.to_string())?;
    }
    // Set nullable based on field_nullable (fixed bug from Python code)
    if field_config.field_nullable {
      builder.with_argument("@Column", "nullable", "true")?;
    } else {
      builder.with_argument("@Column", "nullable", "false")?;
    }
    // Set unique if true
    if field_config.field_unique {
      builder.with_argument("@Column", "unique", "true")?;
    }
    builder.build()
  })
  .ok_or_else(|| "Unable to add new field to the JPA Entity".to_string())?
  .map_err(|e| format!("Unable to add annotations: {}", e))?;
  Ok(())
}

fn parse_entity_file(entity_file_path: &Path) -> Result<TSFile, String> {
  TSFile::from_file(entity_file_path).map_err(|_| "Unable to parse JPA Entity file".to_string())
}

fn save_file(ts_file: &mut TSFile) -> Result<(), String> {
  ts_file.save().map_err(|e| format!("Unable to save JPA Entity file: {}", e))
}

fn build_file_response(ts_file: &TSFile) -> Result<FileResponse, String> {
  let file_type = ts_file.get_file_name_without_ext().unwrap_or_default();
  let file_path = ts_file.file_path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
  let file_package_node = get_package_declaration_node(ts_file)
    .ok_or_else(|| "Unable to get JPA Entity's package node".to_string())?;
  let file_package_scope_node = get_package_class_scope_node(ts_file, file_package_node);
  let file_package_name = file_package_scope_node
    .and_then(|node| ts_file.get_text_from_node(&node))
    .unwrap_or("")
    .to_string();
  Ok(FileResponse { file_type, file_package_name, file_path })
}

pub fn run(
  _cwd: &Path,
  entity_file_path: &Path,
  field_config: EnumFieldConfig,
) -> Result<FileResponse, String> {
  // Step 1: Parse the entity file
  let mut entity_ts_file = parse_entity_file(entity_file_path)?;
  // Step 2: Prepare import map for required imports
  let mut import_map = HashMap::new();
  // Step 3: Add field and annotations to the entity
  add_field_and_annotations(&mut entity_ts_file, &field_config, &mut import_map)?;
  // Step 4: Add all required imports to the file
  add_imports(&mut entity_ts_file, &import_map);
  // Step 5: Write the modified file back to disk
  save_file(&mut entity_ts_file)?;
  // Step 6: Build and return response
  build_file_response(&entity_ts_file)
}
