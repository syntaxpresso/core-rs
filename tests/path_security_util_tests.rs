// Path Security Validator Integration Tests
// This module contains comprehensive tests for the PathSecurityValidator functionality

use std::fs;
use std::path::Path;
use syntaxpresso_core::common::utils::path_security_util::*;
use tempfile::TempDir;

/// Test fixture setup helper
/// Creates a temporary directory with a realistic project structure for testing
pub fn setup_test_directory() -> TempDir {
  let temp_dir = TempDir::new().expect("Failed to create temp directory");

  // Create test directory structure
  fs::create_dir_all(temp_dir.path().join("src")).unwrap();
  fs::create_dir_all(temp_dir.path().join("src/nested")).unwrap();
  fs::create_dir_all(temp_dir.path().join("src/deeply/nested/path")).unwrap();
  fs::write(temp_dir.path().join("src/file.txt"), "test content").unwrap();
  fs::write(temp_dir.path().join("src/nested/deep.txt"), "deep content").unwrap();
  fs::write(temp_dir.path().join("root_file.txt"), "root content").unwrap();

  temp_dir
}

/// Create a test directory with symlinks
/// Creates symlinks for testing symlink resolution and security
#[cfg(unix)]
pub fn setup_test_directory_with_symlinks() -> TempDir {
  let temp_dir = setup_test_directory();

  // Create symlinks for testing
  use std::os::unix::fs::symlink;

  // Safe symlink within base directory
  symlink(temp_dir.path().join("src/file.txt"), temp_dir.path().join("src/safe_symlink.txt"))
    .unwrap();

  // Symlink pointing outside base directory
  symlink("/etc/passwd", temp_dir.path().join("src/malicious_symlink.txt")).unwrap();

  // Broken symlink
  symlink(
    temp_dir.path().join("nonexistent/file.txt"),
    temp_dir.path().join("src/broken_symlink.txt"),
  )
  .unwrap();

  temp_dir
}

// Include all test modules
mod path_security_constructor_tests {
  use super::*;

  #[test]
  fn test_constructor_with_relative_path() {
    // Create a test directory first since constructor requires existing directories
    let temp_dir = setup_test_directory();
    let test_path = temp_dir.path().join("test/path");
    fs::create_dir_all(&test_path).unwrap();

    let validator = PathSecurityValidator::new(&test_path).unwrap();
    assert!(validator.base_path().ends_with("test/path"));
    assert!(validator.base_path().is_absolute());
  }

  #[test]
  fn test_constructor_with_absolute_path() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();
    assert_eq!(validator.base_path(), temp_dir.path().canonicalize().unwrap());
  }

  #[test]
  fn test_constructor_with_nonexistent_path() {
    let temp_dir = setup_test_directory();
    let nonexistent_path = temp_dir.path().join("nonexistent/deeply/nested");
    let result = PathSecurityValidator::new(&nonexistent_path);
    // Constructor requires existing directories in the current implementation
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot canonicalize base path"));
  }

  #[test]
  fn test_constructor_with_file_instead_of_directory() {
    let temp_dir = setup_test_directory();
    let file_path = temp_dir.path().join("src/file.txt");
    let result = PathSecurityValidator::new(&file_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("is not a directory"));
  }

  #[test]
  fn test_constructor_with_current_directory() {
    let current_dir = std::env::current_dir().unwrap();
    let validator = PathSecurityValidator::new(&current_dir).unwrap();
    assert_eq!(validator.base_path(), current_dir);
  }

  #[test]
  fn test_constructor_with_path_containing_spaces() {
    let temp_dir = setup_test_directory();
    let spaced_path = temp_dir.path().join("path with spaces");
    fs::create_dir_all(&spaced_path).unwrap();
    let validator = PathSecurityValidator::new(&spaced_path).unwrap();
    assert!(validator.base_path().to_string_lossy().contains("path with spaces"));
  }
}

mod path_security_validation_tests {
  use super::*;

  #[test]
  fn test_validate_safe_relative_path() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src/file.txt"));
    assert!(result.is_ok());
    let validated_path = result.unwrap();
    assert_eq!(validated_path, temp_dir.path().join("src/file.txt"));
  }

  #[test]
  fn test_validate_safe_absolute_path_within_base() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();
    let safe_absolute_path = temp_dir.path().join("src/nested/deep.txt");

    let result = validator.validate_path_containment(&safe_absolute_path);
    assert!(result.is_ok());
    let validated_path = result.unwrap();
    assert_eq!(validated_path, safe_absolute_path);
  }

  #[test]
  fn test_validate_nonexistent_path() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src/nonexistent/file.txt"));
    assert!(result.is_ok());
    let expected_path = temp_dir.path().join("src/nonexistent/file.txt");
    assert_eq!(result.unwrap(), expected_path);
  }

  #[test]
  fn test_validate_root_path() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("."));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), temp_dir.path());
  }

  #[test]
  fn test_validate_deeply_nested_path() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src/deeply/nested/path/file.txt"));
    assert!(result.is_ok());
    let expected_path = temp_dir.path().join("src/deeply/nested/path/file.txt");
    assert_eq!(result.unwrap(), expected_path);
  }

  #[test]
  fn test_validate_path_with_spaces() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();
    let spaced_dir = temp_dir.path().join("dir with spaces");
    fs::create_dir_all(&spaced_dir).unwrap();

    let result = validator.validate_path_containment(Path::new("dir with spaces/file.txt"));
    assert!(result.is_ok());
    let expected_path = temp_dir.path().join("dir with spaces/file.txt");
    assert_eq!(result.unwrap(), expected_path);
  }

  #[test]
  fn test_validate_unicode_path() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();
    let unicode_dir = temp_dir.path().join("测试目录");
    fs::create_dir_all(&unicode_dir).unwrap();

    let result = validator.validate_path_containment(Path::new("测试目录/文件.txt"));
    assert!(result.is_ok());
    let expected_path = temp_dir.path().join("测试目录/文件.txt");
    assert_eq!(result.unwrap(), expected_path);
  }
}

mod path_security_attack_tests {
  use super::*;

  #[test]
  fn test_prevent_parent_directory_traversal() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(&temp_dir.path().join("src")).unwrap();

    let result = validator.validate_path_containment(Path::new("../root_file.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_prevent_multiple_parent_directory_traversal() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(&temp_dir.path().join("src/nested")).unwrap();

    let result = validator.validate_path_containment(Path::new("../../root_file.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_prevent_absolute_path_outside_base() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("/etc/passwd"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_prevent_mixed_traversal_attack() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(&temp_dir.path().join("src")).unwrap();

    let result = validator.validate_path_containment(Path::new("nested/../../root_file.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_prevent_url_encoded_traversal() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src%2F..%2Froot_file.txt"));
    assert!(result.is_ok()); // URL encoding should be handled by the OS/filesystem
    // The path should be treated literally, not as traversal
  }

  #[test]
  fn test_prevent_double_dot_variations() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(&temp_dir.path().join("src")).unwrap();

    // Test various forms of parent directory references
    let malicious_paths = vec![
      "../root_file.txt",
      "./../root_file.txt",
      "./../../root_file.txt",
      "nested/../../../root_file.txt",
    ];

    for path in malicious_paths {
      let result = validator.validate_path_containment(Path::new(path));
      assert!(result.is_err(), "Path should be rejected: {}", path);
      assert!(result.unwrap_err().contains("outside allowed directory"));
    }
  }

  #[test]
  fn test_allow_legitimate_relative_paths() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let safe_paths = vec![
      "src/file.txt",
      "src/nested/deep.txt",
      "./src/file.txt",
      "src/./file.txt",
      "src/nested/../file.txt", // This resolves to src/file.txt which is safe
    ];

    for path in safe_paths {
      let result = validator.validate_path_containment(Path::new(path));
      assert!(result.is_ok(), "Safe path should be allowed: {}", path);
    }
  }

  #[test]
  fn test_prevent_windows_path_separators_on_unix() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    // Windows-style paths should be handled appropriately
    let result = validator.validate_path_containment(Path::new("src\\file.txt"));
    // On Unix systems, backslashes are valid filename characters
    assert!(result.is_ok());
  }

  #[test]
  fn test_prevent_null_byte_injection() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result =
      validator.validate_path_containment(Path::new("src/file.txt\0../../../etc/passwd"));
    // Paths with null bytes should be rejected as invalid path structure
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid path structure"));
  }

  #[test]
  fn test_complex_traversal_attempt() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(&temp_dir.path().join("src/deeply/nested")).unwrap();

    let result = validator.validate_path_containment(Path::new("../../../root_file.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }
}

#[cfg(unix)]
mod path_security_symlink_tests {
  use super::*;

  #[test]
  fn test_follow_safe_symlink() {
    let temp_dir = setup_test_directory_with_symlinks();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src/safe_symlink.txt"));
    assert!(result.is_ok());
    let resolved_path = result.unwrap();
    assert_eq!(resolved_path, temp_dir.path().join("src/file.txt"));
  }

  #[test]
  fn test_prevent_malicious_symlink() {
    let temp_dir = setup_test_directory_with_symlinks();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src/malicious_symlink.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_handle_broken_symlink() {
    let temp_dir = setup_test_directory_with_symlinks();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src/broken_symlink.txt"));
    assert!(result.is_ok()); // Broken symlinks should still validate if within bounds
    // For broken symlinks, the validator returns the symlink path itself, not the target
    let expected_path = temp_dir.path().join("src/broken_symlink.txt");
    assert_eq!(result.unwrap(), expected_path);
  }

  #[test]
  fn test_symlink_chain_resolution() {
    let temp_dir = setup_test_directory_with_symlinks();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    // Create a chain of symlinks
    use std::os::unix::fs::symlink;
    symlink(
      temp_dir.path().join("src/safe_symlink.txt"),
      temp_dir.path().join("src/chain_symlink.txt"),
    )
    .unwrap();

    let result = validator.validate_path_containment(Path::new("src/chain_symlink.txt"));
    assert!(result.is_ok());
    let resolved_path = result.unwrap();
    assert_eq!(resolved_path, temp_dir.path().join("src/file.txt"));
  }

  #[test]
  fn test_symlink_pointing_to_directory() {
    let temp_dir = setup_test_directory_with_symlinks();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    // Create symlink to directory
    use std::os::unix::fs::symlink;
    symlink(temp_dir.path().join("src"), temp_dir.path().join("src_symlink")).unwrap();

    let result = validator.validate_path_containment(Path::new("src_symlink/file.txt"));
    assert!(result.is_ok());
    let resolved_path = result.unwrap();
    assert_eq!(resolved_path, temp_dir.path().join("src/file.txt"));
  }
}

mod path_security_directory_tests {
  use super::*;

  #[test]
  fn test_validate_directory_creation() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let new_dir_path = Path::new("src/new_directory");
    let result = validator.validate_directory_creation(new_dir_path);
    assert!(result.is_ok());

    let validated_path = result.unwrap();
    assert_eq!(validated_path, temp_dir.path().join(new_dir_path));
  }

  #[test]
  fn test_validate_nested_directory_creation() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let nested_dir_path = Path::new("src/deeply/nested/new/directory");
    let result = validator.validate_directory_creation(nested_dir_path);
    assert!(result.is_ok());

    let validated_path = result.unwrap();
    assert_eq!(validated_path, temp_dir.path().join(nested_dir_path));
  }

  #[test]
  fn test_validate_existing_directory() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let existing_dir_path = Path::new("src");
    let result = validator.validate_directory_creation(existing_dir_path);
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.exists());
    assert!(path.is_dir());
  }

  #[test]
  fn test_prevent_directory_traversal_attack() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(&temp_dir.path().join("src")).unwrap();

    let result = validator.validate_directory_creation(Path::new("../malicious_dir"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_reject_file_as_directory() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    // Try to validate an existing file as a directory
    let result = validator.validate_directory_creation(Path::new("src/file.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exists but is not a directory"));
  }
}

mod path_security_convenience_tests {
  use super::*;

  #[test]
  fn test_validate_path_within_base_success() {
    let temp_dir = setup_test_directory();
    let base_path = temp_dir.path();

    let result = validate_path_within_base(base_path, Path::new("src/file.txt"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), base_path.join("src/file.txt"));
  }

  #[test]
  fn test_validate_path_within_base_traversal() {
    let temp_dir = setup_test_directory();
    let base_path = temp_dir.path().join("src");

    let result = validate_path_within_base(&base_path, Path::new("../root_file.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_validate_path_within_base_absolute_outside() {
    let temp_dir = setup_test_directory();
    let base_path = temp_dir.path();

    let result = validate_path_within_base(base_path, Path::new("/etc/passwd"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_validate_directory_creation_within_base_success() {
    let temp_dir = setup_test_directory();
    let base_path = temp_dir.path();

    let result = validate_directory_creation_within_base(base_path, Path::new("src/new_safe_dir"));
    assert!(result.is_ok());

    let validated_path = result.unwrap();
    assert_eq!(validated_path, base_path.join("src/new_safe_dir"));
  }

  #[test]
  fn test_validate_directory_creation_within_base_traversal() {
    let temp_dir = setup_test_directory();
    let base_path = temp_dir.path().join("src");

    let result = validate_directory_creation_within_base(&base_path, Path::new("../malicious_dir"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("outside allowed directory"));
  }

  #[test]
  fn test_validate_directory_creation_within_base_file_conflict() {
    let temp_dir = setup_test_directory();
    let base_path = temp_dir.path();

    let result = validate_directory_creation_within_base(base_path, Path::new("src/file.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exists but is not a directory"));
  }
}

mod path_security_edge_case_tests {
  use super::*;

  #[test]
  fn test_path_with_consecutive_separators() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src//file.txt"));
    assert!(result.is_ok());
    // The path should be normalized
    assert_eq!(result.unwrap(), temp_dir.path().join("src/file.txt"));
  }

  #[test]
  fn test_path_with_trailing_separator() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("src/"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), temp_dir.path().join("src"));
  }

  #[test]
  fn test_path_with_current_directory_references() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result = validator.validate_path_containment(Path::new("./src/./file.txt"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), temp_dir.path().join("src/file.txt"));
  }

  #[test]
  fn test_very_long_path() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    // Create a very long path (but reasonable)
    let long_path_str = "a".repeat(100) + "/" + &"b".repeat(100) + ".txt";
    let long_path = Path::new(&long_path_str);
    let result = validator.validate_path_containment(long_path);
    assert!(result.is_ok());
  }

  #[test]
  fn test_path_with_special_characters() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    // Test various special characters that are valid in filenames
    let special_paths = vec![
      "file-name.txt",
      "file_name.txt",
      "file name.txt",
      "file.name.txt",
      "file@name.txt",
      "file#name.txt",
      "file$name.txt",
      "file%name.txt",
    ];

    for path_str in special_paths {
      let path = Path::new(path_str);
      let result = validator.validate_path_containment(path);
      assert!(result.is_ok(), "Special character path should be valid: {}", path_str);
    }
  }

  #[test]
  fn test_case_sensitive_paths() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    let result1 = validator.validate_path_containment(Path::new("src/File.txt"));
    let result2 = validator.validate_path_containment(Path::new("src/file.txt"));

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // On case-sensitive filesystems, these should be different
    #[cfg(unix)]
    {
      assert_ne!(result1.unwrap(), result2.unwrap());
    }
  }

  #[test]
  fn test_base_path_with_symlinks() {
    let temp_dir = setup_test_directory();

    // Create symlink to temp directory
    #[cfg(unix)]
    {
      use std::os::unix::fs::symlink;
      let symlink_path =
        temp_dir.path().parent().unwrap().join(format!("temp_symlink_{}", std::process::id()));
      // Remove symlink if it exists from previous runs
      let _ = std::fs::remove_file(&symlink_path);
      symlink(temp_dir.path(), &symlink_path).unwrap();

      let validator = PathSecurityValidator::new(&symlink_path).unwrap();
      let result = validator.validate_path_containment(Path::new("src/file.txt"));
      assert!(result.is_ok());
    }
  }

  #[test]
  fn test_relative_base_path_edge_cases() {
    // Test with various relative path formats
    let current_dir = std::env::current_dir().unwrap();
    let test_dir = current_dir.join("test_relative");
    fs::create_dir_all(&test_dir).unwrap();

    let relative_paths = vec!["./test_relative", "test_relative/.", "test_relative"];

    for rel_path_str in relative_paths {
      let rel_path = Path::new(rel_path_str);
      let result = PathSecurityValidator::new(rel_path);
      assert!(result.is_ok(), "Relative path should be valid: {}", rel_path_str);
    }

    // Clean up
    fs::remove_dir_all(&test_dir).ok();
  }
}

mod path_security_integration_tests {
  use super::*;

  #[test]
  fn test_real_world_project_structure() {
    let temp_dir = setup_test_directory();

    // Create a realistic project structure
    let project_dirs = vec![
      "src/main/java/com/example",
      "src/main/resources",
      "src/test/java/com/example",
      "target/classes",
      "target/test-classes",
    ];

    for dir in &project_dirs {
      fs::create_dir_all(temp_dir.path().join(dir)).unwrap();
    }

    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    // Test various legitimate access patterns
    let legitimate_paths = vec![
      "src/main/java/com/example/MyClass.java",
      "src/main/resources/application.properties",
      "target/classes/com/example/MyClass.class",
      "pom.xml",
      "README.md",
    ];

    for path_str in legitimate_paths {
      let path = Path::new(path_str);
      let result = validator.validate_path_containment(path);
      assert!(result.is_ok(), "Legitimate project path should be allowed: {}", path_str);
    }
  }

  #[test]
  fn test_concurrent_validator_usage() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    // Test that the validator can be used concurrently
    use std::sync::Arc;
    use std::thread;

    let validator = Arc::new(validator);
    let mut handles = vec![];

    for i in 0..10 {
      let validator_clone: Arc<PathSecurityValidator> = Arc::clone(&validator);
      let handle = thread::spawn(move || {
        let path_str = format!("src/file_{}.txt", i);
        let path = Path::new(&path_str);
        validator_clone.validate_path_containment(path)
      });
      handles.push(handle);
    }

    for handle in handles {
      let result = handle.join().unwrap();
      assert!(result.is_ok());
    }
  }

  #[test]
  fn test_performance_with_many_validations() {
    let temp_dir = setup_test_directory();
    let validator = PathSecurityValidator::new(temp_dir.path()).unwrap();

    use std::time::Instant;
    let start = Instant::now();

    // Perform many validations
    for i in 0..1000 {
      let path_str = format!("src/file_{}.txt", i % 100);
      let path = Path::new(&path_str);
      let result = validator.validate_path_containment(path);
      assert!(result.is_ok());
    }

    let duration = start.elapsed();

    // Ensure reasonable performance (should complete well under 1 second)
    assert!(duration.as_millis() < 1000, "Performance test took too long: {:?}", duration);
  }
}
