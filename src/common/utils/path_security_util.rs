use std::fs;
use std::path::{Path, PathBuf};

/// A security validator that ensures all file operations stay within a designated base directory.
/// This prevents path traversal attacks and ensures files can only be created/modified within
/// the allowed directory scope.
#[derive(Debug)]
pub struct PathSecurityValidator {
  base_path: PathBuf,
}

/// Security event types for audit logging
#[derive(Debug)]
pub enum SecurityEvent {
  PathValidationSuccess { target_path: String, base_path: String },
  PathValidationFailure { target_path: String, base_path: String, reason: String },
  PathTraversalAttempt { target_path: String, base_path: String },
  SymlinkDetected { target_path: String, resolved_path: String },
}

/// Log a security event to stderr for audit purposes
/// This provides a simple audit trail without requiring external logging dependencies
/// Note: Only logs failures and security concerns, not successful validations (to avoid noise)
fn log_security_event(event: &SecurityEvent) {
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_else(|_| std::time::Duration::from_secs(0))
    .as_secs();
  match event {
    SecurityEvent::PathValidationSuccess { .. } => {
      // Don't log successful validations to avoid cluttering output
      // Uncomment for debugging: eprintln!("[SECURITY] [{}] Path validation SUCCESS: '{}' within base '{}'", timestamp, target_path, base_path);
    }
    SecurityEvent::PathValidationFailure { target_path, base_path, reason } => {
      eprintln!(
        "[SECURITY] [{}] Path validation FAILURE: '{}' within base '{}' - {}",
        timestamp, target_path, base_path, reason
      );
    }
    SecurityEvent::PathTraversalAttempt { target_path, base_path } => {
      eprintln!(
        "[SECURITY] [{}] PATH TRAVERSAL ATTEMPT BLOCKED: '{}' outside base '{}'",
        timestamp, target_path, base_path
      );
    }
    SecurityEvent::SymlinkDetected { target_path, resolved_path } => {
      eprintln!(
        "[SECURITY] [{}] Symlink detected and resolved: '{}' -> '{}'",
        timestamp, target_path, resolved_path
      );
    }
  }
}

impl PathSecurityValidator {
  /// Creates a new path security validator with the given base path.
  /// The base path is canonicalized to resolve any symbolic links or relative components.
  ///
  /// # Arguments
  /// * `base_path` - The base directory that all operations must be contained within
  ///
  /// # Returns
  /// * `Ok(PathSecurityValidator)` - If the base path is valid and accessible
  /// * `Err(String)` - If the base path cannot be canonicalized or accessed
  ///
  /// # Examples
  /// ```
  /// use std::path::Path;
  /// use syntaxpresso_core::common::utils::path_security_util::PathSecurityValidator;
  ///
  /// # fn main() -> Result<(), String> {
  /// let validator = PathSecurityValidator::new(Path::new("/tmp"))?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn new(base_path: &Path) -> Result<Self, String> {
    let canonical_base = fs::canonicalize(base_path)
      .map_err(|e| format!("Cannot canonicalize base path '{}': {}", base_path.display(), e))?;
    if !canonical_base.is_dir() {
      return Err(format!("Base path '{}' is not a directory", canonical_base.display()));
    }
    Ok(Self { base_path: canonical_base })
  }

  /// Validates that a target path is contained within the base path.
  /// This method prevents path traversal attacks by ensuring the resolved path
  /// is within the allowed directory scope.
  ///
  /// # Arguments
  /// * `target_path` - The path to validate for containment
  ///
  /// # Returns
  /// * `Ok(PathBuf)` - The canonicalized path if it's within the base directory
  /// * `Err(String)` - If the path is outside the base directory or invalid
  ///
  /// # Security Features
  /// - Resolves symbolic links to prevent symlink attacks
  /// - Handles both existing and non-existing paths
  /// - Prevents directory traversal with `../` sequences
  /// - Blocks absolute paths that escape the base directory
  ///
  /// # Examples
  /// ```
  /// use std::path::Path;
  /// use syntaxpresso_core::common::utils::path_security_util::PathSecurityValidator;
  ///
  /// # fn main() -> Result<(), String> {
  /// let validator = PathSecurityValidator::new(Path::new("/tmp"))?;
  /// let safe_path = validator.validate_path_containment(Path::new("src/main.rs"))?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn validate_path_containment(&self, target_path: &Path) -> Result<PathBuf, String> {
    // Handle absolute paths - convert to relative if they're within base
    let working_path = if target_path.is_absolute() {
      if target_path.starts_with(&self.base_path) {
        target_path
          .strip_prefix(&self.base_path)
          .map_err(|_| "Failed to strip base path prefix".to_string())?
          .to_path_buf()
      } else {
        return Err(format!(
          "Absolute path '{}' is outside allowed directory '{}'",
          target_path.display(),
          self.base_path.display()
        ));
      }
    } else {
      target_path.to_path_buf()
    };
    // Build the full path relative to base
    let full_path = self.base_path.join(&working_path);
    // Canonicalize the target path
    let canonical_target = if full_path.exists() {
      // Path exists - canonicalize directly
      fs::canonicalize(&full_path).map_err(|e| {
        format!("Cannot canonicalize existing path '{}': {}", full_path.display(), e)
      })?
    } else {
      // Path doesn't exist yet - canonicalize parent and append filename
      canonicalize_non_existent_path(&full_path)?
    };
    // Verify containment after canonicalization
    if !canonical_target.starts_with(&self.base_path) {
      let event = SecurityEvent::PathTraversalAttempt {
        target_path: target_path.display().to_string(),
        base_path: self.base_path.display().to_string(),
      };
      log_security_event(&event);
      return Err(format!(
        "Path traversal detected: '{}' resolves to '{}' which is outside allowed directory '{}'",
        target_path.display(),
        canonical_target.display(),
        self.base_path.display()
      ));
    }
    // Log successful validation
    let event = SecurityEvent::PathValidationSuccess {
      target_path: target_path.display().to_string(),
      base_path: self.base_path.display().to_string(),
    };
    log_security_event(&event);
    Ok(canonical_target)
  }

  /// Validates that a path intended for directory creation is safe.
  /// This is a specialized version of path validation for directory operations.
  pub fn validate_directory_creation(&self, target_path: &Path) -> Result<PathBuf, String> {
    let validated_path = self.validate_path_containment(target_path)?;
    // Additional check: if the path exists, ensure it's a directory
    if validated_path.exists() && !validated_path.is_dir() {
      return Err(format!("Path '{}' exists but is not a directory", validated_path.display()));
    }
    Ok(validated_path)
  }
  /// Returns the base path used for validation
  pub fn base_path(&self) -> &Path {
    &self.base_path
  }
}

/// Handle canonicalization for non-existent paths by canonicalizing the parent
/// and appending the filename/directory name
pub fn canonicalize_non_existent_path(path: &Path) -> Result<PathBuf, String> {
  if let Some(parent) = path.parent() {
    let canonical_parent = if parent.exists() {
      fs::canonicalize(parent).map_err(|e| {
        format!("Cannot canonicalize parent directory '{}': {}", parent.display(), e)
      })?
    } else {
      // Recursively handle non-existent parents
      canonicalize_non_existent_path(parent)?
    };
    if let Some(filename) = path.file_name() {
      Ok(canonical_parent.join(filename))
    } else {
      Err(format!("Invalid path structure: '{}'", path.display()))
    }
  } else {
    Err(format!("Cannot determine parent directory for path: '{}'", path.display()))
  }
}

/// Convenience function to create a validator and validate a path in one step
pub fn validate_path_within_base(base_path: &Path, target_path: &Path) -> Result<PathBuf, String> {
  let validator = PathSecurityValidator::new(base_path)?;
  validator.validate_path_containment(target_path)
}

/// Convenience function to validate directory creation within a base path
pub fn validate_directory_creation_within_base(
  base_path: &Path,
  target_path: &Path,
) -> Result<PathBuf, String> {
  let validator = PathSecurityValidator::new(base_path)?;
  validator.validate_directory_creation(target_path)
}
