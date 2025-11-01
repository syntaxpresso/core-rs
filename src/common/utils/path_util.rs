#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::common::{
  ts_file::TSFile, types::java_source_directory_type::JavaSourceDirectoryType,
  utils::path_security_util::PathSecurityValidator,
};

/// Recursively searches for a directory with the given name within the root directory.
///
/// # Arguments
/// * `root_dir` - The root directory to search in
/// * `target_dir` - The directory name/path to search for
///
/// # Returns
/// An `Option<PathBuf>` containing the found directory path, or `None` if not found
fn find_directory_recursively(root_dir: &Path, target_dir: &str) -> Option<PathBuf> {
  for entry in WalkDir::new(root_dir).into_iter().flatten() {
    let path = entry.path();
    if path.is_dir() && path.ends_with(target_dir) {
      return Some(path.to_path_buf());
    }
  }
  None
}

pub fn parse_all_files(cwd: &Path, source_directory_type: &JavaSourceDirectoryType) -> Vec<TSFile> {
  let extension = "java";
  let mut files = Vec::new();
  let src_dir_path = source_directory_type.get_directory_path();
  let target_dir = cwd.join(src_dir_path);
  if target_dir.exists() {
    for entry in WalkDir::new(&target_dir).into_iter().flatten() {
      let path = entry.path();
      if let Some(ext) = path.extension()
        && ext.to_string_lossy().eq_ignore_ascii_case(extension)
        && let Ok(ts_file) = TSFile::from_file(path)
      {
        files.push(ts_file);
      }
    }
  }
  files
}

/// Resolves the file system path for a given package scope within the specified source directory type.
///
/// This function finds the appropriate source directory (main or test), converts the package
/// scope (e.g., "com.example.foo") to a directory path, and ensures the directory exists.
///
/// **Security**: This function now includes path traversal protection to ensure all created
/// directories remain within the root directory scope.
///
/// # Arguments
/// * `root_dir` - The root directory of the project
/// * `package_scope` - The package scope as a dot-separated string (e.g., "com.example.foo")
/// * `source_directory_type` - The type of source directory (main or test)
///
/// # Returns
/// A `Result<PathBuf, String>` containing the resolved package directory path, or an error message
///
/// # Security Features
/// - Validates that the resolved path stays within the root directory
/// - Prevents path traversal attacks through package names
/// - Ensures directory creation is safe and contained
///
/// # Examples
/// ```
/// use std::path::Path;
/// use crate::common::types::java_source_directory_type::JavaSourceDirectoryType;
///
/// let project_root = Path::new("/path/to/project");
/// let package_dir = get_file_path_from_package_scope(
///     project_root,
///     "com.example.foo",
///     &JavaSourceDirectoryType::Main
/// )?;
/// // Use package_dir as the directory for new source files
/// ```
pub fn get_file_path_from_package_scope(
  root_dir: &Path,
  package_scope: &str,
  source_directory_type: &JavaSourceDirectoryType,
) -> Result<PathBuf, String> {
  // Basic validation
  if !root_dir.exists() || !root_dir.is_dir() {
    return Err(format!(
      "Root directory does not exist or is not a directory: {}",
      root_dir.display()
    ));
  }
  if package_scope.trim().is_empty() {
    return Err("Package scope cannot be empty".to_string());
  }
  // Create security validator for the root directory
  let validator = PathSecurityValidator::new(root_dir)?;
  // Find or construct the source directory
  let src_dir_name = source_directory_type.get_directory_path();
  let source_dir = find_directory_recursively(root_dir, src_dir_name)
    .unwrap_or_else(|| root_dir.join(src_dir_name));
  // Validate that the source directory is within root (for the constructed case)
  let validated_source_dir = validator.validate_directory_creation(&source_dir)?;
  // Convert package scope to path (e.g., "com.example.foo" -> "com/example/foo")
  let package_as_path = package_scope.replace('.', "/");
  let full_package_dir = validated_source_dir.join(package_as_path);
  // Validate the full package directory path for security
  let validated_package_dir = validator.validate_directory_creation(&full_package_dir)?;
  // Create the directory structure
  match fs::create_dir_all(&validated_package_dir) {
    Ok(_) => Ok(validated_package_dir),
    Err(e) => Err(format!(
      "Failed to create package directory '{}': {}",
      validated_package_dir.display(),
      e
    )),
  }
}
