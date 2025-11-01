use crate::common::utils::path_security_util::PathSecurityValidator;
use std::path::PathBuf;

/// Validates a directory with additional security checks to ensure it's within allowed scope.
/// This function prevents path traversal attacks by validating the directory against the current working directory.
///
/// # Arguments
/// * `s` - The directory path string to validate
/// * `base_path` - Optional base path to validate against. If None, uses current working directory
///
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized, validated directory path
/// * `Err(String)` - If the directory is invalid, doesn't exist, or is outside the allowed scope
///
/// # Security Features
/// - Prevents path traversal attacks (e.g., "../../../etc")
/// - Resolves symbolic links to prevent symlink attacks
/// - Ensures directory stays within the specified base path
///
/// # Examples
/// ```
/// use std::path::Path;
/// use crate::commands::validators::directory_validator::validate_directory_with_security;
///
/// // Validate against current working directory
/// let safe_dir = validate_directory_with_security("src/main/java", None)?;
///
/// // Validate against specific base path
/// let base = Path::new("/project/root");
/// let safe_dir = validate_directory_with_security("src/main/java", Some(base))?;
/// ```
pub fn validate_directory_with_security(
  s: &str,
  base_path: Option<&std::path::Path>,
) -> Result<PathBuf, String> {
  let path = PathBuf::from(s);
  // Basic validation first
  if !path.exists() || !path.is_dir() {
    return Err(format!("Directory does not exist or is not a directory: {}", s));
  }
  // Determine base path for security validation
  let base = match base_path {
    Some(bp) => bp.to_path_buf(),
    None => {
      std::env::current_dir().map_err(|e| format!("Cannot determine current directory: {}", e))?
    }
  };
  // Security validation - ensure the directory is within allowed scope
  let validator = PathSecurityValidator::new(&base)
    .map_err(|e| format!("Security validation setup failed: {}", e))?;
  validator
    .validate_path_containment(&path)
    .map_err(|e| format!("Security validation failed: {}", e))
}

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
/// # Examples
/// ```
/// use std::path::Path;
/// use crate::commands::validators::directory_validator::validate_file_path_within_base;
///
/// let base = Path::new("/project/src");
/// let safe_file = validate_file_path_within_base("entities/User.java", base)?;
/// ```
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

/// Wrapper function for validate_directory_with_security that uses current directory as base.
/// This function is designed for use with clap's value_parser to provide secure directory validation
/// for command line arguments, preventing path traversal attacks on working directory parameters.
///
/// # Arguments
/// * `s` - The directory path string to validate
///
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized, validated directory path
/// * `Err(String)` - If the directory is invalid, doesn't exist, or attempts path traversal
///
/// # Security Features
/// - Uses current working directory as security base to prevent escaping user context
/// - Prevents path traversal attacks (e.g., "../../../etc")
/// - Resolves symbolic links to prevent symlink attacks
/// - Ensures directory stays within allowed scope
///
/// # Usage with clap
/// ```rust
/// #[arg(long, value_parser = validate_directory_secure, required = true)]
/// cwd: PathBuf,
/// ```
pub fn validate_directory_secure(s: &str) -> Result<PathBuf, String> {
  // Use current directory as security base to prevent escaping user's context
  let current_dir =
    std::env::current_dir().map_err(|e| format!("Cannot determine current directory: {}", e))?;
  validate_directory_with_security(s, Some(&current_dir))
}
