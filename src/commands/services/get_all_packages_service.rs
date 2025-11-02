use std::{collections::HashSet, path::Path};

use crate::{
  common::{
    services::package_declaration_service::{get_package_declaration_node, get_package_scope_node},
    types::java_source_directory_type::JavaSourceDirectoryType,
    utils::path_util::parse_all_files,
  },
  responses::package_response::PackageResponse,
};

pub fn run(
  cwd: &Path,
  source_directory: &JavaSourceDirectoryType,
) -> Result<HashSet<PackageResponse>, String> {
  let mut response: HashSet<PackageResponse> = HashSet::new();
  let ts_files = parse_all_files(cwd, source_directory);
  for ts_file in ts_files {
    match get_package_declaration_node(&ts_file) {
      Some(package_declaration_node) => {
        let package_scope_node = get_package_scope_node(&ts_file, package_declaration_node);
        if package_scope_node.is_none() {
          continue;
        }
        let package_name = ts_file.get_text_from_node(&package_scope_node.unwrap());
        if package_name.is_none() {
          continue;
        }
        let package = PackageResponse { package_name: String::from(package_name.unwrap()) };
        response.insert(package);
      }
      None => continue,
    }
  }
  Ok(response)
}
