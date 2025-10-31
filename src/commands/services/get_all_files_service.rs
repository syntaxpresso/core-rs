use std::path::Path;

use crate::{
  common::{
    services::package_declaration_service::{get_package_declaration_node, get_package_scope_node},
    utils::path_util::parse_all_files,
  },
  responses::file_response::FileResponse,
};

pub fn run(cwd: &Path) -> Result<Vec<FileResponse>, String> {
  let mut files = Vec::new();
  let ts_files = parse_all_files(cwd);
  for ts_file in ts_files {
    let file_type = ts_file.get_file_name_without_ext().unwrap_or_else(|| "Unknown".to_string());
    let file_path = ts_file
      .file_path()
      .map(|p| p.to_string_lossy().to_string())
      .unwrap_or_else(|| "Unknown path".to_string());
    let file_package_name = if let Some(package_node) = get_package_declaration_node(&ts_file) {
      get_package_scope_node(&ts_file, package_node)
        .and_then(|name_node| ts_file.get_text_from_node(&name_node))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "No package".to_string())
    } else {
      "No package".to_string()
    };
    let found_file = FileResponse { file_type, file_package_name, file_path };
    files.push(found_file);
  }
  Ok(files)
}
