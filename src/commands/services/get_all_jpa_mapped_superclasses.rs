use std::path::Path;

use crate::{
  common::{
    services::{
      annotation_service::find_annotation_node_by_name,
      class_declaration_service::get_public_class_node,
      package_declaration_service::{get_package_declaration_node, get_package_scope_node},
    },
    types::java_source_directory_type::JavaSourceDirectoryType,
    utils::path_util::parse_all_files,
  },
  responses::file_response::FileResponse,
};

pub fn run(cwd: &Path) -> Result<Vec<FileResponse>, String> {
  let mut files: Vec<FileResponse> = Vec::new();
  let ts_files = parse_all_files(cwd, &JavaSourceDirectoryType::Main);
  for ts_file in ts_files {
    match get_public_class_node(&ts_file) {
      Some(public_class_node) => {
        let mapped_superclass_annotation_node =
          find_annotation_node_by_name(&ts_file, public_class_node, "MappedSuperclass");
        if mapped_superclass_annotation_node.is_some() {
          let file_type =
            ts_file.get_file_name_without_ext().unwrap_or_else(|| "Unknown".to_string());
          let file_path = ts_file
            .file_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown path".to_string());
          let file_package_name = if let Some(package_node) = get_package_declaration_node(&ts_file)
          {
            get_package_scope_node(&ts_file, package_node)
              .and_then(|name_node| ts_file.get_text_from_node(&name_node))
              .map(|s| s.to_string())
              .unwrap_or_else(|| "No package".to_string())
          } else {
            continue;
          };
          let found_file = FileResponse { file_type, file_package_name, file_path };
          files.push(found_file);
        }
      }
      None => continue,
    }
  }
  Ok(files)
}
