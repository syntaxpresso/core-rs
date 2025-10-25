#![allow(dead_code)]

use crate::common::ts_file::TSFile;
use tree_sitter::Node;

/// Retrieves the package declaration node from a Java source file.
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
///
/// # Returns
/// * `Some(Node)` - The package declaration node if found
/// * `None` - If no package declaration exists in the file
///
/// # Example
/// ```rust
/// // For Java file containing: package com.example.myapp;
/// let package_node = get_package_declaration_node(&ts_file);
/// // Returns the entire package_declaration node
/// ```
pub fn get_package_declaration_node(ts_file: &TSFile) -> Option<Node<'_>> {
    ts_file.tree.as_ref()?;
    let query_string = "(package_declaration) @package";
    ts_file.query_first_node(query_string, "package")
}

/// Extracts the complete package name as a string from a package declaration.
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
/// * `_package_node` - The package declaration node (currently unused, kept for API consistency)
///
/// # Returns
/// * `Some(String)` - The full package name
/// * `None` - If no package name could be extracted
///
/// # Example
/// ```rust
/// // For Java file containing: package com.example.myapp;
/// let package_name = get_package_name(&ts_file, &package_node);
/// // Returns: Some("com.example.myapp".to_string())
/// ```
///
/// # Expected Result
/// Returns the complete package identifier as text (e.g., "com.example.myapp")
pub fn get_package_name(ts_file: &TSFile, _package_node: &Node) -> Option<String> {
    // Simpler approach: query for any identifier or scoped_identifier in package_declaration
    let query_string = r#"
        (package_declaration 
          (scoped_identifier) @name)
    "#;
    // Find the name part of the package declaration
    if let Some(name_node) = ts_file.query_first_node(query_string, "name") {
        ts_file
            .get_text_from_node(&name_node)
            .map(|s| s.to_string())
    } else {
        // Fallback: try simple identifier for single-part package names
        let simple_query = r#"
            (package_declaration 
              (identifier) @name)
        "#;
        if let Some(name_node) = ts_file.query_first_node(simple_query, "name") {
            ts_file
                .get_text_from_node(&name_node)
                .map(|s| s.to_string())
        } else {
            None
        }
    }
}

/// Extracts the rightmost part (class name) from a package declaration.
/// This is typically used to get the final component of a package name,
/// which often represents the application or module name.
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
/// * `_package_declaration_node` - The package declaration node (currently unused)
///
/// # Returns
/// * `Some(Node)` - The node containing the class name part
/// * `None` - If no scoped identifier is found or package has only one component
///
/// # Example
/// ```rust
/// // For Java file containing: package com.example.myapp;
/// let class_name_node = get_package_class_name_node(&ts_file, &package_node);
/// let class_name_text = ts_file.get_text_from_node(&class_name_node.unwrap());
/// // Returns node containing: "myapp"
/// ```
///
/// # Expected Result
/// Returns the rightmost identifier from the package declaration (e.g., "myapp" from "com.example.myapp")
pub fn get_package_class_name_node<'a>(
    ts_file: &'a TSFile,
    _package_declaration_node: &Node,
) -> Option<Node<'a>> {
    let query_string = r#"
        (package_declaration
          (scoped_identifier
            name: (_) @class_name
          )
        )
    "#;
    ts_file.query_first_node(query_string, "class_name")
}

/// Extracts the scope part (everything except the rightmost component) from a package declaration.
/// This represents the organizational hierarchy excluding the final application/module name.
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
/// * `_package_declaration_node` - The package declaration node (currently unused)
///
/// # Returns
/// * `Some(Node)` - The node containing the scope part
/// * `None` - If no scoped identifier is found or package has only one component
///
/// # Example
/// ```rust
/// // For Java file containing: package com.example.myapp;
/// let scope_node = get_package_class_scope_node(&ts_file, &package_node);
/// let scope_text = ts_file.get_text_from_node(&scope_node.unwrap());
/// // Returns node containing: "com.example"
/// ```
///
/// # Expected Result
/// Returns everything except the rightmost identifier (e.g., "com.example" from "com.example.myapp")
pub fn get_package_class_scope_node<'a>(
    ts_file: &'a TSFile,
    _package_declaration_node: &Node,
) -> Option<Node<'a>> {
    let query_string = r#"
        (package_declaration
          (scoped_identifier
            scope: (_) @class_scope
          )
        )
    "#;
    ts_file.query_first_node(query_string, "class_scope")
}

/// Extracts the complete scoped identifier node from a package declaration.
/// This returns the entire dotted package name as a single node.
///
/// # Arguments
/// * `ts_file` - Reference to the TSFile containing the parsed Java code
/// * `_package_declaration_node` - The package declaration node (currently unused)
///
/// # Returns
/// * `Some(Node)` - The scoped identifier node containing the full package name
/// * `None` - If no scoped identifier is found
///
/// # Example
/// ```rust
/// // For Java file containing: package com.example.myapp;
/// let scope_node = get_package_scope_node(&ts_file, &package_node);
/// let scope_text = ts_file.get_text_from_node(&scope_node.unwrap());
/// // Returns node containing: "com.example.myapp"
/// ```
///
/// # Expected Result
/// Returns the complete dotted package identifier (e.g., "com.example.myapp")
pub fn get_package_scope_node<'a>(
    ts_file: &'a TSFile,
    _package_declaration_node: &Node,
) -> Option<Node<'a>> {
    let query_string = r#"
        (package_declaration
          (scoped_identifier) @package_scope
        )
    "#;
    ts_file.query_first_node(query_string, "package_scope")
}
