#![allow(dead_code)]

use crate::common::services::package_declaration_service::get_package_declaration_node;
use crate::common::ts_file::TSFile;
use crate::common::types::import_types::{ImportInsertionPoint, ImportInsertionPosition};
use tree_sitter::Node;

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

pub fn add_import<'a>(
    ts_file: &'a mut TSFile,
    insertion_position: &ImportInsertionPosition,
    import_package_scope: &str,
    import_class: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none()
        || import_package_scope.trim().is_empty()
        || import_class.trim().is_empty()
    {
        return None;
    }
    if find_import_declaration_node(ts_file, import_package_scope, import_class).is_some() {
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
        let formated_import_text = format!("import {}.{};", import_package_scope, import_class);
        match (
            import_insertion_point.break_line_before,
            import_insertion_point.break_line_after,
        ) {
            (true, true) => format!("{}\n\n{}{}", before, formated_import_text, after),
            (true, false) => format!("{}\n{}{}", before, formated_import_text, after),
            (false, true) => format!("{}{}\n{}", before, formated_import_text, after),
            (false, false) => format!("{}{}{}", before, formated_import_text, after),
        }
    };
    // Replace the entire file content with the new content
    ts_file.replace_text_by_byte_range(0, file_content.len(), &new_content)
}
