pub fn validate_package_name(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        return Err("Package name cannot be empty".to_string());
    }
    if !s
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_')
    {
        return Err(
            "Package name can only contain letters, numbers, dots, and underscores".to_string(),
        );
    }
    if s.starts_with('.') || s.ends_with('.') || s.contains("..") {
        return Err(
            "Package name cannot start/end with dots or contain consecutive dots".to_string(),
        );
    }
    Ok(s.to_string())
}
