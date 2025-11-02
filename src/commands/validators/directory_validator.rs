use crate::common::utils::path_security_util::PathSecurityValidator;
use std::path::PathBuf;

/// Validates a file path with security checks to ensure it's within the specified base directory.
/// This function prevents path traversal attacks and ensures files can only be accessed within allowed scope.
///
/// # Arguments
/// * `file_path` - The file path string to validate
/// * `base_path` - The base directory that the file must be contained within
///
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized, validated file path
/// * `Err(String)` - If the file path is invalid or outside the allowed scope
///
/// # Security Features
/// - Prevents path traversal attacks (e.g., "../../../etc/passwd")
/// - Resolves symbolic links to prevent symlink attacks
/// - Ensures file path stays within the specified base directory
/// - Handles both existing and non-existent files
///
pub fn validate_file_path_within_base(
  file_path: &str,
  base_path: &std::path::Path,
) -> Result<PathBuf, String> {
  let path = PathBuf::from(file_path);
  // Security validation - ensure the file path is within allowed scope
  let validator = PathSecurityValidator::new(base_path)
    .map_err(|e| format!("Security validation setup failed: {}", e))?;
  validator
    .validate_path_containment(&path)
    .map_err(|e| format!("File path security validation failed: {}", e))
}

/// Validates that a directory exists and is accessible, without security restrictions.
/// This function is designed for the --cwd parameter to allow users to work on any project
/// they have access to. Security validation happens WITHIN the chosen directory, not on the directory itself.
///
/// # Arguments
/// * `s` - The directory path string to validate
///
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized directory path
/// * `Err(String)` - If the directory is invalid or doesn't exist
///
/// # Security Philosophy
/// - The --cwd parameter should accept any valid directory (user's project root)
/// - Security restrictions apply to operations WITHIN the chosen cwd, not to the cwd selection itself
/// - Users should be able to work on projects anywhere they have filesystem access
pub fn validate_directory_unrestricted(s: &str) -> Result<PathBuf, String> {
  let path = PathBuf::from(s);
  // Basic validation - ensure directory exists and is accessible
  if !path.exists() {
    return Err(format!("Directory does not exist: {}", s));
  }
  if !path.is_dir() {
    return Err(format!("Path is not a directory: {}", s));
  }
  // Canonicalize to resolve any symbolic links and get absolute path
  path.canonicalize().map_err(|e| format!("Cannot canonicalize directory path '{}': {}", s, e))
}
