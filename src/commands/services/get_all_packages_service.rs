use std::{collections::HashSet, path::Path};

use walkdir::WalkDir;

use crate::{
  common::{
    services::package_declaration_service::{get_package_declaration_node, get_package_scope_node},
    types::java_source_directory_type::JavaSourceDirectoryType,
    utils::path_util::parse_all_files,
  },
  responses::package_response::PackageResponse,
};

/// Finds the root package by analyzing the directory structure.
///
/// The heuristic looks for the first directory level that contains:
/// - Java files, OR
/// - Multiple subdirectories (indicating project structure branching)
///
/// Example: For org.example.demo project:
/// - src/main/java/org -> single subdir, skip
/// - src/main/java/org/example -> single subdir, skip
/// - src/main/java/org/example/demo -> has Java files OR multiple subdirs = ROOT!
fn find_root_package_from_structure(
  cwd: &Path,
  source_directory: &JavaSourceDirectoryType,
) -> Option<String> {
  let src_dir_path = cwd.join(source_directory.get_directory_path());
  if !src_dir_path.exists() {
    return None;
  }
  // Collect all directories with their depth
  let mut directories: Vec<_> = WalkDir::new(&src_dir_path)
    .min_depth(1)
    .into_iter()
    .flatten()
    .filter(|e| e.path().is_dir())
    .collect();
  // Sort by depth (shallowest first)
  directories.sort_by_key(|e| e.depth());
  for entry in directories {
    let dir_path = entry.path();
    // Check if this directory contains Java files
    let has_java_files = std::fs::read_dir(dir_path).ok()?.flatten().any(|e| {
      e.path().is_file()
        && e
          .path()
          .extension()
          .and_then(|ext| ext.to_str())
          .is_some_and(|ext| ext.eq_ignore_ascii_case("java"))
    });
    // Count immediate subdirectories
    let subdir_count =
      std::fs::read_dir(dir_path).ok()?.flatten().filter(|e| e.path().is_dir()).count();
    // Root package heuristic: has Java files OR multiple subdirectories
    if (has_java_files || subdir_count > 1)
      && let Ok(relative_path) = dir_path.strip_prefix(&src_dir_path)
    {
      let package_name = relative_path.to_string_lossy().replace(['\\', '/'], ".");
      return Some(package_name);
    }
  }
  None
}

pub fn run(
  cwd: &Path,
  source_directory: &JavaSourceDirectoryType,
) -> Result<HashSet<PackageResponse>, String> {
  let mut response: HashSet<PackageResponse> = HashSet::new();
  // Step 1: Try to find root package from existing Java files
  let ts_files = parse_all_files(cwd, source_directory);
  let root_package = if !ts_files.is_empty() {
    // Collect all packages from files
    let mut packages: Vec<String> = Vec::new();
    for ts_file in &ts_files {
      if let Some(package_declaration_node) = get_package_declaration_node(ts_file)
        && let Some(package_scope_node) = get_package_scope_node(ts_file, package_declaration_node)
        && let Some(package_name) = ts_file.get_text_from_node(&package_scope_node)
      {
        packages.push(String::from(package_name));
      }
    }
    // Find the shortest package (= root package)
    packages.into_iter().min_by_key(|pkg| pkg.split('.').count())
  } else {
    // Step 2: Fallback - infer root package from directory structure
    find_root_package_from_structure(cwd, source_directory)
  };
  // If we found a root package, discover all packages from it
  if let Some(root_pkg) = root_package {
    let src_dir_path = cwd.join(source_directory.get_directory_path());
    let root_pkg_path = root_pkg.replace('.', "/");
    let root_package_dir = src_dir_path.join(&root_pkg_path);
    // Add the root package itself
    response.insert(PackageResponse { package_name: root_pkg.clone() });
    // Traverse all subdirectories starting from root package
    if root_package_dir.exists() {
      for entry in WalkDir::new(&root_package_dir).min_depth(1).into_iter().flatten() {
        if entry.path().is_dir()
          && let Ok(relative_path) = entry.path().strip_prefix(&src_dir_path)
        {
          let package_name = relative_path.to_string_lossy().replace(['\\', '/'], ".");
          response.insert(PackageResponse { package_name });
        }
      }
    }
  }
  Ok(response)
}
