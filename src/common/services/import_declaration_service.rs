#![allow(dead_code)]

use crate::common::services::package_declaration_service::get_package_declaration_node;
use crate::common::ts_file::TSFile;
use crate::common::types::import_types::ImportInsertionPosition;
use tree_sitter::Node;

pub fn get_all_import_declaration_nodes<'a>(ts_file: &'a TSFile) -> Vec<Node<'a>> {
    if ts_file.tree.is_none() {
        return Vec::new();
    }
    let query_string = r#"
        (import_declaration) @declaration
    "#;
    match ts_file
        .query_builder(query_string)
        .returning("declaration")
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

/// Retrieves the relative import scope node from an import declaration.
///
/// This method extracts the package portion of an import statement. For example, in
/// `import java.util.List;`, it would return the node representing "java.util".
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
/// * `import_declaration_node` - The import declaration node to analyze
///
/// # Returns
/// Optional containing the relative import scope node, empty if not found
///
/// # Example
/// ```
/// // For Java file containing: import java.util.List;
/// let imports = get_all_import_declaration_nodes(&ts_file);
/// if let Some(scope_node) = get_import_declaration_relative_import_scope_node(&ts_file, imports[0]) {
///     let scope_text = ts_file.get_text_from_node(&scope_node);
///     // scope_text = Some("java.util")
/// }
/// ```
pub fn get_import_declaration_relative_import_scope_node<'a>(
    ts_file: &'a TSFile,
    import_declaration_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || import_declaration_node.kind() != "import_declaration" {
        return None;
    }
    let query_string = r#"
        (import_declaration
          [
            (scoped_identifier) @full_import_scope
            (scoped_identifier
              scope: (_) @relative_import_scope
              name: (_) @class_name)
          ]
          (asterisk)? @asterisk
        ) @import
    "#;
    ts_file
        .query_builder(query_string)
        .within(import_declaration_node)
        .returning("relative_import_scope")
        .execute()
        .ok()?
        .first_node()
}

/// Retrieves the class name node from an import declaration.
///
/// This method extracts the class name portion of an import statement. For example, in
/// `import java.util.List;`, it would return the node representing "List".
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
/// * `import_declaration_node` - The import declaration node to analyze
///
/// # Returns
/// Optional containing the class name node, empty if not found or if it's a wildcard import
///
/// # Example
/// ```
/// // For Java file containing: import java.util.List;
/// let imports = get_all_import_declaration_nodes(&ts_file);
/// if let Some(class_name_node) = get_import_declaration_class_name_node(&ts_file, imports[0]) {
///     let class_name = ts_file.get_text_from_node(&class_name_node);
///     // class_name = Some("List")
/// }
/// ```
pub fn get_import_declaration_class_name_node<'a>(
    ts_file: &'a TSFile,
    import_declaration_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || import_declaration_node.kind() != "import_declaration" {
        return None;
    }
    let query_string = r#"
        (import_declaration
          [
            (scoped_identifier) @full_import_scope
            (scoped_identifier
              scope: (_) @relative_import_scope
              name: (_) @class_name)
          ]
          (asterisk)? @asterisk
        ) @import
    "#;
    ts_file
        .query_builder(query_string)
        .within(import_declaration_node)
        .returning("class_name")
        .execute()
        .ok()?
        .first_node()
}

/// Retrieves the full import scope node from an import declaration.
///
/// This method extracts the complete import path from an import statement. For regular imports
/// like `import java.util.List;`, it returns the entire "java.util.List". For wildcard
/// imports like `import java.util.*;`, it returns "java.util".
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
/// * `import_declaration_node` - The import declaration node to analyze
///
/// # Returns
/// Optional containing the full import scope node, empty if not found
///
/// # Example
/// ```
/// // For Java file containing: import java.util.List;
/// let imports = get_all_import_declaration_nodes(&ts_file);
/// if let Some(full_scope_node) = get_import_declaration_full_import_scope_node(&ts_file, imports[0]) {
///     let full_scope = ts_file.get_text_from_node(&full_scope_node);
///     // full_scope = Some("java.util.List") for regular imports
///     // full_scope = Some("java.util") for wildcard imports
/// }
/// ```
pub fn get_import_declaration_full_import_scope_node<'a>(
    ts_file: &'a TSFile,
    import_declaration_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || import_declaration_node.kind() != "import_declaration" {
        return None;
    }
    let query_string = r#"
        (import_declaration
          [
            (scoped_identifier) @full_import_scope
            (scoped_identifier
              scope: (_) @relative_import_scope
              name: (_) @class_name)
          ]
          (asterisk)? @asterisk
        ) @import
    "#;
    ts_file
        .query_builder(query_string)
        .within(import_declaration_node)
        .returning("full_import_scope")
        .execute()
        .ok()?
        .first_node()
}

/// Finds the import declaration node for a given package scope and class name.
///
/// This method searches for an import declaration that matches the specified package scope and
/// class name. It handles both regular imports (exact class match) and wildcard imports (package
/// scope match with asterisk).
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
/// * `package_scope` - The package scope to search for (e.g., "java.util")
/// * `class_name` - The class name to search for (e.g., "List")
///
/// # Returns
/// Optional containing the matching import declaration node, empty if not found or invalid input
///
/// # Example
/// ```
/// // For Java file containing: import java.util.List; or import java.util.*;
/// if let Some(import_node) = find_import_declaration_node(&ts_file, "java.util", "List") {
///     let import_text = ts_file.get_text_from_node(&import_node);
///     // import_text = Some("import java.util.List;") or covered by "import java.util.*;"
/// }
/// ```
pub fn find_import_declaration_node<'a>(
    ts_file: &'a TSFile,
    package_scope: &str,
    class_name: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || class_name.is_empty() || package_scope.is_empty() {
        return None;
    }
    let all_imports = get_all_import_declaration_nodes(ts_file);
    for import_declaration_node in all_imports {
        // Check for wildcard imports first using the full import scope
        if let Some(full_import_node) =
            get_import_declaration_full_import_scope_node(ts_file, import_declaration_node)
            && let Some(full_import_text) = ts_file.get_text_from_node(&full_import_node)
        {
            // Check if this import has an asterisk (wildcard)
            let query_string = r#"(import_declaration (asterisk) @asterisk)"#;
            let has_asterisk = ts_file
                .query_builder(query_string)
                .within(import_declaration_node)
                .returning("asterisk")
                .execute()
                .is_ok_and(|result| result.first_node().is_some());
            if has_asterisk {
                // For wildcard imports, check if the full import scope matches the package scope
                if package_scope == full_import_text {
                    return Some(import_declaration_node);
                }
            }
        }
        // Check for regular imports using relative scope and class name
        if let Some(scope_node) =
            get_import_declaration_relative_import_scope_node(ts_file, import_declaration_node)
            && let Some(class_node) =
                get_import_declaration_class_name_node(ts_file, import_declaration_node)
        {
            let scope_text = ts_file.get_text_from_node(&scope_node);
            let class_text = ts_file.get_text_from_node(&class_node);
            if let (Some(scope), Some(class)) = (scope_text, class_text)
                && package_scope == scope
                && class_name == class
            {
                return Some(import_declaration_node);
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
struct ImportInsertionPoint {
    position: ImportInsertionPosition,
    insert_byte: usize,
    break_line_before: bool,
    break_line_after: bool,
}

impl ImportInsertionPoint {
    fn new() -> Self {
        Self {
            position: ImportInsertionPosition::AfterLastImport,
            insert_byte: 0,
            break_line_before: false,
            break_line_after: false,
        }
    }
}

/// Adds an import declaration to a Java file at the specified insertion position.
///
/// This method inserts a new import statement at the appropriate location based on the
/// insertion position. It handles proper formatting with line breaks and follows Java
/// import organization conventions.
///
/// # Arguments
/// * `ts_file` - Mutable reference to the TSFile to modify
/// * `insertion_position` - Where to insert the import (before first, after last, or after package)
/// * `import_text` - The complete import statement text (e.g., "import java.util.List;")
///
/// # Returns
/// Optional containing the updated root node after insertion, empty if insertion failed
///
/// # Example
/// ```
/// // Add import after existing imports
/// if let Some(updated_node) = add_import(
///     &mut ts_file,
///     &ImportInsertionPosition::AfterLastImport,
///     "import java.util.HashMap;"
/// ) {
///     // Import successfully added
/// }
/// ```
pub fn add_import<'a>(
    ts_file: &'a mut TSFile,
    insertion_position: &ImportInsertionPosition,
    import_text: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || import_text.trim().is_empty() {
        return None;
    }
    // Get the root node to work with the entire file
    let root_node = ts_file.tree.as_ref()?.root_node();
    let file_content = ts_file.get_text_from_node(&root_node)?.to_string();
    // Collect all necessary information before any mutable operations
    let (package_declaration_node, all_imports) = {
        let package_declaration_node = get_package_declaration_node(ts_file);
        let all_imports = get_all_import_declaration_nodes(ts_file);
        (package_declaration_node, all_imports)
    };
    let mut import_insertion_point = ImportInsertionPoint::new();
    import_insertion_point.position = insertion_position.clone();
    // Determine insertion point based on position
    match insertion_position {
        ImportInsertionPosition::AfterPackageDeclaration => {
            if let Some(package_node) = package_declaration_node {
                import_insertion_point.break_line_before = true;
                import_insertion_point.break_line_after = true;
                import_insertion_point.insert_byte = package_node.end_byte();
            } else {
                // No package declaration, insert at beginning of file
                import_insertion_point.break_line_after = true;
                import_insertion_point.insert_byte = 0;
            }
        }
        ImportInsertionPosition::BeforeFirstImport => {
            if !all_imports.is_empty() {
                import_insertion_point.break_line_after = true;
                import_insertion_point.insert_byte = all_imports[0].start_byte();
            } else if let Some(package_node) = package_declaration_node {
                // No imports but package exists, insert after package
                import_insertion_point.break_line_before = true;
                import_insertion_point.break_line_after = true;
                import_insertion_point.insert_byte = package_node.end_byte();
            } else {
                // No package and no imports, insert at beginning
                import_insertion_point.break_line_after = true;
                import_insertion_point.insert_byte = 0;
            }
        }
        ImportInsertionPosition::AfterLastImport => {
            if !all_imports.is_empty() {
                import_insertion_point.break_line_before = true;
                import_insertion_point.insert_byte = all_imports.last()?.end_byte();
            } else if let Some(package_node) = package_declaration_node {
                // No imports but package exists, insert after package
                import_insertion_point.break_line_before = true;
                import_insertion_point.break_line_after = true;
                import_insertion_point.insert_byte = package_node.end_byte();
            } else {
                // No package and no imports, insert at beginning
                import_insertion_point.break_line_after = true;
                import_insertion_point.insert_byte = 0;
            }
        }
    }
    // Build the new content string with proper formatting
    let new_content = {
        let insertion_byte = import_insertion_point.insert_byte;
        let before = &file_content[..insertion_byte];
        let after = &file_content[insertion_byte..];
        match (
            import_insertion_point.break_line_before,
            import_insertion_point.break_line_after,
        ) {
            (true, true) => format!("{}\n\n{}{}", before, import_text, after),
            (true, false) => format!("{}\n{}{}", before, import_text, after),
            (false, true) => format!("{}{}\n{}", before, import_text, after),
            (false, false) => format!("{}{}{}", before, import_text, after),
        }
    };
    // Replace the entire file content with the new content
    ts_file.replace_text_by_byte_range(0, file_content.len(), &new_content)
}
