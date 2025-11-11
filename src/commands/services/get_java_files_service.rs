use std::path::Path;

use crate::{
  common::{
    services::{
      annotation_type_declaration_service::get_public_annotation_type_node,
      class_declaration_service::get_public_class_node,
      enum_declaration_service::get_public_enum_node,
      interface_declaration_service::get_public_interface_node,
      package_declaration_service::{get_package_declaration_node, get_package_scope_node},
      record_declaration_service::get_public_record_node,
    },
    ts_file::TSFile,
    types::{java_file_type::JavaFileType, java_source_directory_type::JavaSourceDirectoryType},
    utils::path_util::parse_all_files,
  },
  responses::file_response::FileResponse,
};

fn create_file_response(ts_file: &TSFile) -> Option<FileResponse> {
  let file_type = ts_file.get_file_name_without_ext().unwrap_or_else(|| "Unknown".to_string());
  let file_path = ts_file
    .file_path()
    .map(|p| p.to_string_lossy().to_string())
    .unwrap_or_else(|| "Unknown path".to_string());
  let file_package_name = if let Some(package_node) = get_package_declaration_node(ts_file) {
    get_package_scope_node(ts_file, package_node)
      .and_then(|name_node| ts_file.get_text_from_node(&name_node))
      .map(|s| s.to_string())
      .unwrap_or_else(|| "No package".to_string())
  } else {
    return None;
  };
  let found_file = FileResponse { file_type, file_package_name, file_path };
  Some(found_file)
}

pub fn run(cwd: &Path, java_file_type: &JavaFileType) -> Result<Vec<FileResponse>, String> {
  let mut files: Vec<FileResponse> = Vec::new();
  let ts_files = parse_all_files(cwd, &JavaSourceDirectoryType::Main);
  for ts_file in ts_files {
    match java_file_type {
      JavaFileType::Class => match get_public_class_node(&ts_file) {
        // TODO: is get_public_class_node falling back to non public classes?
        Some(_) => {
          if let Some(file_response) = create_file_response(&ts_file) {
            files.push(file_response);
          }
        }
        None => continue,
      },
      JavaFileType::Interface => match get_public_interface_node(&ts_file) {
        Some(_) => {
          if let Some(file_response) = create_file_response(&ts_file) {
            files.push(file_response);
          }
        }
        None => continue,
      },
      JavaFileType::Enum => match get_public_enum_node(&ts_file) {
        Some(_) => {
          if let Some(file_response) = create_file_response(&ts_file) {
            files.push(file_response);
          }
        }
        None => continue,
      },
      JavaFileType::Annotation => match get_public_annotation_type_node(&ts_file) {
        Some(_) => {
          if let Some(file_response) = create_file_response(&ts_file) {
            files.push(file_response);
          }
        }
        None => continue,
      },
      JavaFileType::Record => match get_public_record_node(&ts_file) {
        Some(_) => {
          if let Some(file_response) = create_file_response(&ts_file) {
            files.push(file_response);
          }
        }
        None => continue,
      },
    }
  }
  Ok(files)
}
