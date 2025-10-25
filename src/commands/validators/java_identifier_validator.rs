pub fn validate_java_identifier(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        return Err("Java identifier cannot be empty".to_string());
    }
    let first_char = s.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return Err("Java identifier must start with a letter or underscore".to_string());
    }
    if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(
            "Java identifier can only contain letters, numbers, and underscores".to_string(),
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
    if reserved_words.contains(&s) {
        return Err(format!(
            "'{}' is a Java reserved word and cannot be used as an identifier",
            s
        ));
    }
    Ok(s.to_string())
}

