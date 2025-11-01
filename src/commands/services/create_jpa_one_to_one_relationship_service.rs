use crate::common::services::class_declaration_service::get_public_class_node;
use crate::common::services::field_declaration_service::{
  AddFieldDeclarationParams, add_field_declaration,
};
use crate::common::services::import_declaration_service::add_import;
use crate::common::services::package_declaration_service::{
  get_package_declaration_node, get_package_scope_node,
};
use crate::common::ts_file::TSFile;
use crate::common::types::annotation_config::AnnotationConfig;
use crate::common::types::cascade_type::CascadeType;
use crate::common::types::entity_side::EntitySide;
use crate::common::types::field_types::FieldInsertionPosition;
use crate::common::types::import_types::ImportInsertionPosition;
use crate::common::types::java_source_directory_type::JavaSourceDirectoryType;
use crate::common::types::java_visibility_modifier::JavaVisibilityModifier;
use crate::common::types::mapping_type::MappingType;
use crate::common::types::one_to_one_field_config::OneToOneFieldConfig;
use crate::common::types::other_type::OtherType;
use crate::common::types::processed_imports::ProcessedImports;
use crate::common::utils::case_util::{self, CaseType};
use crate::common::utils::path_util::parse_all_files;
use crate::responses::file_response::FileResponse;
use std::collections::HashMap;
use std::path::{Path, PathBuf};



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

fn find_inverse_entity(cwd: &Path, class_name: &str) -> Result<PathBuf, String> {
  let ts_files = parse_all_files(cwd, &JavaSourceDirectoryType::Main);
  for ts_file in ts_files {
    if let Some(file_name) = ts_file.get_file_name_without_ext()
      && file_name == class_name
      && let Some(public_class_node) = get_public_class_node(&ts_file)
    {
      use crate::common::services::annotation_service::find_annotation_node_by_name;
      if find_annotation_node_by_name(&ts_file, public_class_node, "Entity").is_some()
        && let Some(file_path) = ts_file.file_path()
      {
        return Ok(file_path.to_path_buf());
      }
    }
  }
  Err(format!("Entity class '{}' not found in project", class_name))
}

fn extract_owning_entity_class_name(file_path: &Path) -> Result<String, String> {
  file_path
    .file_stem()
    .and_then(|stem| stem.to_str())
    .map(|s| s.to_string())
    .ok_or_else(|| format!("Unable to extract class name from path: {}", file_path.display()))
}

fn get_entity_package_name(entity_file_path: &Path) -> Result<String, String> {
  let ts_file = TSFile::from_file(entity_file_path)
    .map_err(|_| "Unable to parse entity file for package name".to_string())?;
  let package_node = get_package_declaration_node(&ts_file)
    .ok_or_else(|| "Unable to get package declaration from entity".to_string())?;
  let package_scope_node = get_package_scope_node(&ts_file, package_node);
  package_scope_node
    .and_then(|node| ts_file.get_text_from_node(&node))
    .map(|s| s.to_string())
    .ok_or_else(|| "Unable to extract package name from entity".to_string())
}

fn build_annotation_config(
  field_config: &OneToOneFieldConfig,
  side: EntitySide,
  mapped_by_field_name: Option<String>,
) -> AnnotationConfig {
  let (cascades, other_options) = match side {
    EntitySide::Owning => {
      (field_config.owning_side_cascades.clone(), field_config.owning_side_other.clone())
    }
    EntitySide::Inverse => {
      (field_config.inverse_side_cascades.clone(), field_config.inverse_side_other.clone())
    }
  };
  let is_owning_side = side == EntitySide::Owning;
  let is_unidirectional = field_config.mapping_type == Some(MappingType::UnidirectionalJoinColumn);
  AnnotationConfig::new_one_to_one(
    is_owning_side,
    cascades,
    other_options,
    if is_owning_side || is_unidirectional { None } else { mapped_by_field_name },
    is_owning_side || is_unidirectional,
  )
}

fn process_imports(
  _field_config: &OneToOneFieldConfig,
  target_entity_type: &str,
  target_entity_file_path: &Path,
  annotation_config: &AnnotationConfig,
) -> Result<ProcessedImports, String> {
  let mut jpa_imports = Vec::new();
  jpa_imports.push(("jakarta.persistence".to_string(), "OneToOne".to_string()));
  if annotation_config.needs_join_column {
    jpa_imports.push(("jakarta.persistence".to_string(), "JoinColumn".to_string()));
  }
  if !annotation_config.cascades.is_empty() {
    jpa_imports.push(("jakarta.persistence".to_string(), "CascadeType".to_string()));
  }
  let target_entity_package = get_entity_package_name(target_entity_file_path)?;
  let mut processed_imports = ProcessedImports::new();
  processed_imports.set_entity_import(target_entity_package, target_entity_type.to_string());
  for (package, class_name) in jpa_imports {
    processed_imports.add_jpa_import(package, class_name);
  }
  Ok(processed_imports)
}

fn build_cascade_param(cascades: &[CascadeType]) -> Option<String> {
  if cascades.is_empty() {
    return None;
  }
  if cascades.len() == 5 {
    return Some("CascadeType.ALL".to_string());
  }
  if cascades.len() == 1 {
    return Some(format!("CascadeType.{}", cascades[0].as_str()));
  }
  let cascade_list =
    cascades.iter().map(|c| format!("CascadeType.{}", c.as_str())).collect::<Vec<_>>().join(", ");
  Some(format!("{{{}}}", cascade_list))
}

fn add_relationship_field(
  ts_file: &mut TSFile,
  field_name: &str,
  target_entity_type: &str,
  annotation_config: &AnnotationConfig,
) -> Result<(), String> {
  let public_class_node = get_public_class_node(ts_file)
    .ok_or_else(|| "Unable to get public class node from Entity".to_string())?;
  let public_class_node_start_byte = public_class_node.start_byte();
  let field_name_camel_case = case_util::auto_convert_case(field_name, CaseType::Camel);
  let target_field_name_snake_case =
    case_util::auto_convert_case(target_entity_type, CaseType::Snake);
  let params = AddFieldDeclarationParams {
    insertion_position: FieldInsertionPosition::EndOfClassBody,
    visibility_modifier: JavaVisibilityModifier::Private,
    field_modifiers: vec![],
    field_type: target_entity_type,
    field_name: &field_name_camel_case,
    field_initialization: None,
  };
  add_field_declaration(ts_file, public_class_node_start_byte, params, |builder| {
    builder.add_annotation("@OneToOne")?;
    if let Some(cascade_param) = build_cascade_param(&annotation_config.cascades) {
      builder.with_argument("@OneToOne", "cascade", &cascade_param)?;
    }
    let is_mandatory = annotation_config.other_options.contains(&OtherType::Mandatory);
    if !is_mandatory {
      builder.with_argument("@OneToOne", "optional", "true")?;
    } else {
      builder.with_argument("@OneToOne", "optional", "false")?;
    }
    if annotation_config.other_options.contains(&OtherType::OrphanRemoval) {
      builder.with_argument("@OneToOne", "orphanRemoval", "true")?;
    }
    if let Some(ref mapped_by_field) = annotation_config.mapped_by_field {
      let mapped_by_snake_case = case_util::auto_convert_case(mapped_by_field, CaseType::Snake);
      builder.with_argument("@OneToOne", "mappedBy", &format!("\"{}\"", mapped_by_snake_case))?;
    }
    if annotation_config.needs_join_column {
      builder.add_annotation("@JoinColumn")?;
      let column_name = format!("{}_id", target_field_name_snake_case);
      builder.with_argument("@JoinColumn", "name", &format!("\"{}\"", column_name))?;
      if is_mandatory {
        builder.with_argument("@JoinColumn", "nullable", "false")?;
      } else {
        builder.with_argument("@JoinColumn", "nullable", "true")?;
      }
      if annotation_config.other_options.contains(&OtherType::Unique) {
        builder.with_argument("@JoinColumn", "unique", "true")?;
      }
    }
    builder.build()
  })
  .ok_or_else(|| "Unable to add relationship field to the JPA Entity".to_string())?
  .map_err(|e| format!("Unable to add annotations: {}", e))?;
  Ok(())
}

fn is_bidirectional_mapping(field_config: &OneToOneFieldConfig) -> bool {
  field_config.mapping_type != Some(MappingType::UnidirectionalJoinColumn)
}

fn process_owning_side_entity(
  entity_file_path: &Path,
  field_name: &str,
  target_entity_type: &str,
  target_entity_file_path: &Path,
  field_config: &OneToOneFieldConfig,
) -> Result<FileResponse, String> {
  process_entity_side(
    entity_file_path,
    field_name,
    target_entity_type,
    target_entity_file_path,
    field_config,
    EntitySide::Owning,
    None, // Owning side doesn't use mappedBy
  )
}

fn process_inverse_side_entity(
  entity_file_path: &Path,
  field_name: &str,
  target_entity_type: &str,
  target_entity_file_path: &Path,
  field_config: &OneToOneFieldConfig,
  mapped_by_field_name: Option<String>,
) -> Result<FileResponse, String> {
  process_entity_side(
    entity_file_path,
    field_name,
    target_entity_type,
    target_entity_file_path,
    field_config,
    EntitySide::Inverse,
    mapped_by_field_name,
  )
}

fn process_entity_side(
  entity_file_path: &Path,
  field_name: &str,
  target_entity_type: &str,
  target_entity_file_path: &Path,
  field_config: &OneToOneFieldConfig,
  side: EntitySide,
  mapped_by_field_name: Option<String>,
) -> Result<FileResponse, String> {
  let mut entity_ts_file = TSFile::from_file(entity_file_path)
    .map_err(|_| "Unable to parse JPA Entity file".to_string())?;
  let annotation_config = build_annotation_config(field_config, side, mapped_by_field_name);
  let processed_imports =
    process_imports(field_config, target_entity_type, target_entity_file_path, &annotation_config)?;
  let mut import_map = HashMap::new();
  for (package, class_name) in processed_imports.jpa_imports {
    add_to_import_map(&mut import_map, &package, &class_name);
  }
  if let Some((package, class_name)) = processed_imports.entity_class_import {
    add_to_import_map(&mut import_map, &package, &class_name);
  }
  add_relationship_field(&mut entity_ts_file, field_name, target_entity_type, &annotation_config)?;
  add_imports(&mut entity_ts_file, &import_map);
  entity_ts_file.save().map_err(|e| format!("Unable to save JPA Entity file: {}", e))?;
  build_file_response(&entity_ts_file)
}

fn build_file_response(ts_file: &TSFile) -> Result<FileResponse, String> {
  let file_type = ts_file.get_file_name_without_ext().unwrap_or_default();
  let file_path = ts_file.file_path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
  let file_package_node = get_package_declaration_node(ts_file)
    .ok_or_else(|| "Unable to get JPA Entity's package node".to_string())?;
  let file_package_scope_node = get_package_scope_node(ts_file, file_package_node);
  let file_package_name = file_package_scope_node
    .and_then(|node| ts_file.get_text_from_node(&node))
    .unwrap_or("")
    .to_string();
  Ok(FileResponse { file_type, file_package_name, file_path })
}

pub fn run(
  cwd: &Path,
  owning_side_entity_file_path: &Path,
  owning_side_field_name: &str,
  inverse_side_field_name: &str,
  field_config: &OneToOneFieldConfig,
) -> Result<Vec<FileResponse>, String> {
  // Step 1: Find inverse entity by class name
  let inverse_entity_file_path = find_inverse_entity(cwd, &field_config.inverse_field_type)?;
  // Step 2: Extract owning entity class name for inverse side
  let owning_entity_class_name = extract_owning_entity_class_name(owning_side_entity_file_path)?;
  // Step 3: Process owning side entity
  let owning_response = process_owning_side_entity(
    owning_side_entity_file_path,
    owning_side_field_name,
    &field_config.inverse_field_type,
    &inverse_entity_file_path,
    field_config,
  )?;
  let mut responses = vec![owning_response];
  // Step 4: Process inverse side entity (if bidirectional)
  if is_bidirectional_mapping(field_config) {
    let inverse_response = process_inverse_side_entity(
      &inverse_entity_file_path,
      inverse_side_field_name,
      &owning_entity_class_name,
      owning_side_entity_file_path,
      field_config,
      Some(owning_side_field_name.to_string()),
    )?;
    responses.push(inverse_response);
  }
  // Step 5: Return processed file responses
  Ok(responses)
}
