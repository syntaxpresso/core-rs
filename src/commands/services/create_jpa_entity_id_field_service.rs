use crate::common::services::class_declaration_service::get_public_class_node;
use crate::common::services::field_declaration_service::{
  AddFieldDeclarationParams, add_field_declaration,
};
use crate::common::services::import_declaration_service::add_import;
use crate::common::services::package_declaration_service::{
  get_package_class_scope_node, get_package_declaration_node,
};
use crate::common::ts_file::TSFile;
use crate::common::types::field_types::FieldInsertionPosition;
use crate::common::types::id_field_config::IdFieldConfig;
use crate::common::types::import_types::ImportInsertionPosition;
use crate::common::types::java_id_generation::JavaIdGeneration;
use crate::common::types::java_id_generation_type::JavaIdGenerationType;
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
  field_config: &IdFieldConfig,
  import_map: &mut HashMap<String, String>,
) -> Result<(), String> {
  let column_name_snake_case =
    case_util::auto_convert_case(&field_config.field_name, CaseType::Snake);
  let _field_name_pascal_case =
    case_util::auto_convert_case(&field_config.field_name, CaseType::Pascal);
  let public_class_node = get_public_class_node(ts_file)
    .ok_or_else(|| "Unable to get public class node from Entity".to_string())?;
  let public_class_node_start_byte = public_class_node.start_byte();
  // Add required imports based on field configuration
  add_to_import_map(import_map, "jakarta.persistence", "Column");
  add_to_import_map(import_map, "jakarta.persistence", "Id");
  add_to_import_map(import_map, "jakarta.persistence", "GeneratedValue");
  add_to_import_map(import_map, "jakarta.persistence", "GenerationType");
  // Add field type import if it has a package
  if let Some(ref package_name) = field_config.field_type_package_name {
    add_to_import_map(import_map, package_name, &field_config.field_type);
  }
  let params = AddFieldDeclarationParams {
    insertion_position: FieldInsertionPosition::EndOfClassBody,
    visibility_modifier: JavaVisibilityModifier::Private,
    field_modifiers: vec![],
    field_type: &field_config.field_type,
    field_name: &field_config.field_name,
    field_initialization: None,
  };
  add_field_declaration(ts_file, public_class_node_start_byte, params, |builder| {
    // Always add @Id annotation
    builder.add_annotation("@Id")?;
    // Add @GeneratedValue if generation is not None
    if field_config.field_id_generation.ne(&JavaIdGeneration::None) {
      builder.add_annotation("@GeneratedValue")?.with_argument(
        "@GeneratedValue",
        "strategy",
        &format!("GenerationType.{}", field_config.field_id_generation.as_str()),
      )?;
      // Handle sequence generation with entity exclusive generation
      if field_config.field_id_generation.eq(&JavaIdGeneration::Sequence)
        && field_config
          .field_id_generation_type
          .eq(&JavaIdGenerationType::EntityExclusiveGeneration)
      {
        add_to_import_map(import_map, "jakarta.persistence", "SequenceGenerator");
        if let Some(ref generator_name) = field_config.field_generator_name {
          builder.with_argument(
            "@GeneratedValue",
            "generator",
            &format!("\"{}\"", generator_name),
          )?;
          // Add @SequenceGenerator annotation
          builder.add_annotation("@SequenceGenerator")?.with_argument(
            "@SequenceGenerator",
            "name",
            &format!("\"{}\"", generator_name),
          )?;
          if let Some(ref sequence_name) = field_config.field_sequence_name {
            builder.with_argument(
              "@SequenceGenerator",
              "sequenceName",
              &format!("\"{}\"", sequence_name),
            )?;
          }
          // Add initialValue if different from default (1)
          if let Some(initial_value) = field_config.field_initial_value
            && initial_value != 1
          {
            builder.with_argument(
              "@SequenceGenerator",
              "initialValue",
              &initial_value.to_string(),
            )?;
          }
          // Add allocationSize if different from default (50)
          if let Some(allocation_size) = field_config.field_allocation_size
            && allocation_size != 50
          {
            builder.with_argument(
              "@SequenceGenerator",
              "allocationSize",
              &allocation_size.to_string(),
            )?;
          }
        }
      }
    }
    // Add @Column annotation
    builder.add_annotation("@Column")?.with_argument(
      "@Column",
      "name",
      &format!("\"{}\"", &column_name_snake_case),
    )?;
    // Set nullable based on field_nullable
    if field_config.field_nullable {
      builder.with_argument("@Column", "nullable", "true")?;
    } else {
      builder.with_argument("@Column", "nullable", "false")?;
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
  field_config: IdFieldConfig,
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
