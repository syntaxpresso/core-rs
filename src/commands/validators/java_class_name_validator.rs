pub fn validate_java_class_name(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        return Err("Class name cannot be empty".to_string());
    }
    // Allow various case formats (snake_case, camelCase, PascalCase, kebab-case, etc.)
    // We'll convert them internally to PascalCase for Java conventions
    if !s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(
            "Class name can only contain letters, numbers, underscores, and hyphens".to_string()
        );
    }
    let reserved_words = [
        "abstract",
        "assert",
        "boolean",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "class",
        "const",
        "continue",
        "default",
        "do",
        "double",
        "else",
        "enum",
        "extends",
        "final",
        "finally",
        "float",
        "for",
        "goto",
        "if",
        "implements",
        "import",
        "instanceof",
        "int",
        "interface",
        "long",
        "native",
        "new",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "short",
        "static",
        "strictfp",
        "super",
        "switch",
        "synchronized",
        "this",
        "throw",
        "throws",
        "transient",
        "try",
        "void",
        "volatile",
        "while",
        "true",
        "false",
        "null",
    ];
    if reserved_words.contains(&s.to_lowercase().as_str()) {
        return Err(format!("'{}' conflicts with a Java reserved word", s));
    }
    if s.contains("__") {
        return Err("Class name cannot contain consecutive underscores".to_string());
    }
    if s.starts_with('_') || s.ends_with('_') {
        return Err("Class name cannot start or end with underscore".to_string());
    }
    Ok(s.to_string())
}
