#![allow(dead_code)]

use crate::common::services::annotation_service::add_annotation;
use crate::common::ts_file::TSFile;
use crate::common::types::annotation_types::AnnotationInsertionPosition;
use crate::common::types::field_types::{FieldInsertionPoint, FieldInsertionPosition};
use crate::common::types::java_field_modifier::JavaFieldModifier;
use crate::common::types::java_visibility_modifier::JavaVisibilityModifier;
use tree_sitter::Node;

pub struct AddFieldDeclarationParams<'a> {
    pub insertion_position: FieldInsertionPosition,
    pub visibility_modifier: JavaVisibilityModifier,
    pub field_modifiers: Vec<JavaFieldModifier>,
    pub field_type: &'a str,
    pub field_name: &'a str,
    pub field_initialization: Option<&'a str>,
}

pub struct FieldAnnotationBuilder<'a> {
    ts_file: &'a mut TSFile,
    field_start_byte: usize,
    pending_annotations: Vec<PendingAnnotation>,
}

struct PendingAnnotation {
    annotation_text: String,
    arguments: Vec<(String, String)>,
}

impl<'a> FieldAnnotationBuilder<'a> {
    pub fn new(ts_file: &'a mut TSFile, field_start_byte: usize) -> Self {
        Self { ts_file, field_start_byte, pending_annotations: Vec::new() }
    }

    pub fn add_annotation(&mut self, annotation_text: &str) -> Result<&mut Self, String> {
        self.pending_annotations.push(PendingAnnotation {
            annotation_text: annotation_text.to_string(),
            arguments: Vec::new(),
        });
        Ok(self)
    }

    fn find_field_declaration_node(&self) -> Option<Node<'_>> {
        // Start from the byte position and traverse up to find field_declaration
        let mut current_node =
            self.ts_file.get_named_node_at_byte_position(self.field_start_byte)?;
        // If we're already at a field_declaration, return it
        if current_node.kind() == "field_declaration" {
            return Some(current_node);
        }
        // Traverse up the parent chain to find field_declaration
        while let Some(parent) = current_node.parent() {
            if parent.kind() == "field_declaration" {
                return Some(parent);
            }
            current_node = parent;
        }
        None
    }

    pub fn with_argument(
        &mut self,
        annotation_text: &str,
        key: &str,
        value: &str,
    ) -> Result<&mut Self, String> {
        // Find the pending annotation and add the argument to it
        let pending_annotation = self
            .pending_annotations
            .iter_mut()
            .find(|pa| pa.annotation_text == annotation_text)
            .ok_or_else(|| format!("No pending annotation found for: {}", annotation_text))?;

        pending_annotation.arguments.push((key.to_string(), value.to_string()));
        Ok(self)
    }

    pub fn with_value(&mut self, annotation_text: &str, value: &str) -> Result<&mut Self, String> {
        // Find the pending annotation and add the single value to it
        let pending_annotation = self
            .pending_annotations
            .iter_mut()
            .find(|pa| pa.annotation_text == annotation_text)
            .ok_or_else(|| format!("No pending annotation found for: {}", annotation_text))?;
        // For single value annotations, we store it as a special argument
        pending_annotation.arguments.push(("__single_value__".to_string(), value.to_string()));
        Ok(self)
    }

    pub fn build(&mut self) -> Result<(), String> {
        // Find the actual field_declaration node from the byte position
        let field_node =
            self.find_field_declaration_node().ok_or("Failed to find field declaration node")?;
        let field_start_byte = field_node.start_byte();
        // Process each pending annotation
        for pending_annotation in &self.pending_annotations {
            // Build the complete annotation text with all arguments
            let mut annotation_text = pending_annotation.annotation_text.clone();
            if !pending_annotation.arguments.is_empty() {
                // Check if this is a single value annotation
                if pending_annotation.arguments.len() == 1
                    && pending_annotation.arguments[0].0 == "__single_value__"
                {
                    let value = &pending_annotation.arguments[0].1;
                    annotation_text = format!("{}({})", annotation_text, value);
                } else {
                    let args_str = pending_annotation
                        .arguments
                        .iter()
                        .filter(|(key, _)| key != "__single_value__")
                        .map(|(key, value)| format!("{} = {}", key, value))
                        .collect::<Vec<_>>()
                        .join(", ");
                    annotation_text = format!("{}({})", annotation_text, args_str);
                }
            }
            // Add the complete annotation to the tree
            add_annotation(
                self.ts_file,
                field_start_byte,
                &AnnotationInsertionPosition::AboveScopeDeclaration,
                &annotation_text,
            )
            .ok_or_else(|| format!("Failed to add annotation: {}", annotation_text))?;
        }
        // Clear pending annotations after building
        self.pending_annotations.clear();
        Ok(())
    }
}

pub fn get_all_field_declaration_nodes<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Vec<Node<'a>> {
    if ts_file.tree.is_none() || scope_node.kind() != "class_declaration" {
        return Vec::new();
    }
    let query_string = r#"
        (class_declaration
          body: (class_body
            (field_declaration) @fieldDeclaration
          )
        )
    "#;
    match ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("fieldDeclaration")
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

pub fn find_field_declaration_node_by_name<'a>(
    ts_file: &'a TSFile,
    field_declarator_name: &str,
    scope_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none()
        || field_declarator_name.trim().is_empty()
        || scope_node.kind() != "class_declaration"
    {
        return None;
    }
    let query_string = format!(
        r#"
        ((field_declaration
          declarator: (variable_declarator
            name: (identifier) @name))
         (#eq? @name "{}")) @fieldDeclaration
        "#,
        field_declarator_name
    );
    ts_file
        .query_builder(&query_string)
        .within(scope_node)
        .returning("fieldDeclaration")
        .execute()
        .ok()?
        .first_node()
}

pub fn find_field_declaration_nodes_by_type<'a>(
    ts_file: &'a TSFile,
    field_declarator_type: &str,
    scope_node: Node<'a>,
) -> Vec<Node<'a>> {
    if ts_file.tree.is_none()
        || field_declarator_type.trim().is_empty()
        || scope_node.kind() != "class_declaration"
    {
        return Vec::new();
    }
    let query_string = format!(
        r#"
        (field_declaration
          type: [
            (type_identifier) @type
            (integral_type) @type
            (floating_point_type) @type
            (boolean_type) @type
            (void_type) @type
            (generic_type
              (type_arguments
                [
                  (type_identifier) @type
                  (integral_type) @type
                  (floating_point_type) @type
                  (boolean_type) @type
                ]
              )
            )
          ]
        ) @fieldDeclaration
        (#eq? @type "{}")
        "#,
        field_declarator_type
    );
    match ts_file
        .query_builder(&query_string)
        .within(scope_node)
        .returning("fieldDeclaration")
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

pub fn get_field_declaration_full_type_node<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || scope_node.kind() != "field_declaration" {
        return None;
    }
    let query_string = r#"
        (field_declaration
          type: [
            (type_identifier) @fullType
            (integral_type) @fullType
            (floating_point_type) @fullType
            (boolean_type) @fullType
            (void_type) @fullType
            (generic_type
              (type_identifier)
              (type_arguments
                [
                  (type_identifier) @fullType
                  (integral_type) @fullType
                  (floating_point_type) @fullType
                  (boolean_type) @fullType
                  (generic_type) @fullType
                ]
              )
            ) @fullType
          ]
        )
    "#;
    ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("fullType")
        .execute()
        .ok()?
        .first_node()
}

pub fn get_field_declaration_type_node<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || scope_node.kind() != "field_declaration" {
        return None;
    }
    let query_string = r#"
        (field_declaration
          type: [
            (type_identifier) @type
            (integral_type) @type
            (floating_point_type) @type
            (boolean_type) @type
            (void_type) @type
          ]
        )
    "#;
    ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("type")
        .execute()
        .ok()?
        .first_node()
}

pub fn get_field_declaration_name_node<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || scope_node.kind() != "field_declaration" {
        return None;
    }
    let query_string = r#"
        (field_declaration
          declarator: (variable_declarator
            name: (identifier) @name
          )
        )
    "#;
    ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("name")
        .execute()
        .ok()?
        .first_node()
}

pub fn get_field_declaration_value_node<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || scope_node.kind() != "field_declaration" {
        return None;
    }
    let query_string = r#"
        (field_declaration
          declarator: (variable_declarator
            name: (identifier)
            value: (_) @value
          )
        )
    "#;
    ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("value")
        .execute()
        .ok()?
        .first_node()
}

pub fn get_all_field_declaration_usage_nodes<'a>(
    ts_file: &'a TSFile,
    field_declaration_node: Node<'a>,
    class_declaration_node: Node<'a>,
) -> Vec<Node<'a>> {
    if ts_file.tree.is_none()
        || field_declaration_node.kind() != "field_declaration"
        || class_declaration_node.kind() != "class_declaration"
    {
        return Vec::new();
    }
    let field_name_node = get_field_declaration_name_node(ts_file, field_declaration_node);
    let field_name_node = match field_name_node {
        Some(node) => node,
        None => return Vec::new(),
    };
    let field_name = match ts_file.get_text_from_node(&field_name_node) {
        Some(name) => name,
        None => return Vec::new(),
    };
    let query_string = format!(
        r#"
        (field_access
          field: (identifier) @usage)
        (#eq? @usage "{}")
        "#,
        field_name
    );
    match ts_file
        .query_builder(&query_string)
        .within(class_declaration_node)
        .returning("usage")
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

pub fn get_all_method_declaration_nodes<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Vec<Node<'a>> {
    if ts_file.tree.is_none() || scope_node.kind() != "class_declaration" {
        return Vec::new();
    }
    let query_string = r#"
        (class_declaration
          body: (class_body
            (method_declaration) @methodDeclaration
          )
        )
    "#;
    match ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("methodDeclaration")
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

pub fn get_class_body_node<'a>(
    ts_file: &'a TSFile,
    class_declaration_node: Node<'a>,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || class_declaration_node.kind() != "class_declaration" {
        return None;
    }
    let query_string = r#"
        (class_declaration
          body: (class_body) @classBody
        )
    "#;
    ts_file
        .query_builder(query_string)
        .within(class_declaration_node)
        .returning("classBody")
        .execute()
        .ok()?
        .first_node()
}

pub fn get_field_insertion_position<'a>(
    ts_file: &'a TSFile,
    class_declaration_node: Node<'a>,
    insertion_position: &FieldInsertionPosition,
) -> Option<FieldInsertionPoint> {
    if ts_file.tree.is_none() || class_declaration_node.kind() != "class_declaration" {
        return None;
    }
    let all_fields = get_all_field_declaration_nodes(ts_file, class_declaration_node);
    let all_methods = get_all_method_declaration_nodes(ts_file, class_declaration_node);
    let mut field_insertion_point = FieldInsertionPoint::new();
    field_insertion_point.position = insertion_position.clone();
    match insertion_position {
        FieldInsertionPosition::AfterLastField => {
            if !all_fields.is_empty() {
                field_insertion_point.break_line_before = true;
                field_insertion_point.insert_byte = all_fields.last()?.end_byte();
            } else {
                // No fields exist, insert at beginning of class body with proper formatting
                let class_body = get_class_body_node(ts_file, class_declaration_node)?;
                field_insertion_point.break_line_before = true;
                field_insertion_point.break_line_after = true;
                field_insertion_point.insert_byte = class_body.start_byte() + 1; // After opening brace
            }
        }
        FieldInsertionPosition::BeforeFirstMethod => {
            if !all_methods.is_empty() {
                field_insertion_point.break_line_after = true;
                field_insertion_point.insert_byte = all_methods.first()?.start_byte();
            } else {
                // No methods exist, fallback to end of class body
                let class_body = get_class_body_node(ts_file, class_declaration_node)?;
                field_insertion_point.break_line_before = true;
                field_insertion_point.insert_byte = class_body.end_byte() - 1; // Before closing brace
            }
        }
        FieldInsertionPosition::EndOfClassBody => {
            let class_body = get_class_body_node(ts_file, class_declaration_node)?;
            field_insertion_point.break_line_before = true;
            field_insertion_point.insert_byte = class_body.end_byte() - 1; // Before closing brace
        }
    }

    Some(field_insertion_point)
}

/// Robustly finds the class declaration node from a given byte position,
/// ascending from annotations or modifiers if necessary.
fn find_class_declaration_node_from_position<'a>(
    ts_file: &'a TSFile,
    byte_position: usize,
) -> Option<Node<'a>> {
    let mut node = ts_file.get_named_node_at_byte_position(byte_position)?;
    // Handle various node types we might encounter
    match node.kind() {
        "class_declaration" => Some(node),
        "modifiers" => node.parent().filter(|p| p.kind() == "class_declaration"),
        "marker_annotation" | "annotation" => {
            let mut current = Some(node);
            while let Some(current_node) = current {
                if current_node.kind() == "class_declaration" {
                    return Some(current_node);
                }
                current = current_node.parent();
            }
            None
        }
        _ => {
            // For other node types, try to find class_declaration in the parent chain
            let mut current = Some(node);
            while let Some(current_node) = current {
                if current_node.kind() == "class_declaration" {
                    node = current_node;
                    break;
                }
                current = current_node.parent();
            }
            if node.kind() == "class_declaration" { Some(node) } else { None }
        }
    }
}

pub fn add_field_declaration<'a, F, R>(
    ts_file: &'a mut TSFile,
    class_declaration_byte_position: usize,
    params: AddFieldDeclarationParams<'a>,
    callback: F,
) -> Option<R>
where
    F: FnOnce(&mut FieldAnnotationBuilder<'a>) -> R,
{
    if ts_file.tree.is_none()
        || params.field_type.trim().is_empty()
        || params.field_name.trim().is_empty()
    {
        return None;
    }
    // Find the class declaration node
    let class_declaration_node =
        find_class_declaration_node_from_position(ts_file, class_declaration_byte_position)?;
    // Get the class body node
    let class_body_node = get_class_body_node(ts_file, class_declaration_node)?;
    // Collect all necessary information before any mutable operations
    let (class_body_start_byte, class_body_end_byte, current_body_text, all_fields) = {
        let current_body_text = ts_file.get_text_from_node(&class_body_node)?.to_string();
        let all_fields = get_all_field_declaration_nodes(ts_file, class_declaration_node);
        (class_body_node.start_byte(), class_body_node.end_byte(), current_body_text, all_fields)
    };
    // Build the field declaration text
    let modifiers_str =
        params.field_modifiers.iter().map(|m| m.keyword()).collect::<Vec<_>>().join(" ");
    let mut field_text = String::new();
    field_text.push_str("  "); // Indentation
    if params.visibility_modifier.has_keyword() {
        field_text.push_str(params.visibility_modifier.keyword());
        field_text.push(' ');
    }
    if !modifiers_str.is_empty() {
        field_text.push_str(&modifiers_str);
        field_text.push(' ');
    }
    field_text.push_str(params.field_type);
    field_text.push(' ');
    field_text.push_str(params.field_name);
    if let Some(field_init) = params.field_initialization
        && !field_init.trim().is_empty()
    {
        field_text.push_str(" = ");
        field_text.push_str(field_init);
    }
    field_text.push(';');
    // Build the new class body content with the field inserted at the proper position
    let new_body_content = match params.insertion_position {
        FieldInsertionPosition::AfterLastField => {
            if !all_fields.is_empty() {
                // Insert after last field
                let last_field = all_fields.last()?;
                let relative_pos = last_field.end_byte() - class_body_start_byte;
                let before = &current_body_text[..relative_pos];
                let after = &current_body_text[relative_pos..];
                format!("{}\n{}{}", before, field_text, after)
            } else {
                // No fields exist, insert after opening brace and before any content
                // The class body text includes braces, so we need to find the right position
                if let Some(after_brace) = current_body_text.strip_prefix('{') {
                    format!("{{\n{}{}", field_text, after_brace)
                } else {
                    current_body_text
                }
            }
        }
        FieldInsertionPosition::BeforeFirstMethod => {
            // For now, use same logic as AfterLastField
            // TODO: Implement method detection if needed
            if !all_fields.is_empty() {
                let last_field = all_fields.last()?;
                let relative_pos = last_field.end_byte() - class_body_start_byte;
                let before = &current_body_text[..relative_pos];
                let after = &current_body_text[relative_pos..];
                format!("{}\n{}{}", before, field_text, after)
            } else if let Some(after_brace) = current_body_text.strip_prefix('{') {
                format!("{{\n{}{}", field_text, after_brace)
            } else {
                current_body_text
            }
        }
        FieldInsertionPosition::EndOfClassBody => {
            // Insert before the closing brace
            if let Some(before_brace) = current_body_text.strip_suffix('}') {
                format!("{}\n{}\n}}", before_brace, field_text)
            } else {
                format!("{}\n{}\n", current_body_text, field_text)
            }
        }
    };
    // Replace the class body with the new content - tree is updated incrementally
    let update_success = ts_file.replace_text_by_byte_range(
        class_body_start_byte,
        class_body_end_byte,
        &new_body_content,
    );
    if update_success.is_some() {
        // The tree is updated. We must get the class node again from the *new* tree.
        let new_class_decl_node =
            find_class_declaration_node_from_position(ts_file, class_declaration_byte_position)?;
        // Now, find the new field node we just added
        let new_field_node =
            find_field_declaration_node_by_name(ts_file, params.field_name, new_class_decl_node)?;
        let field_start_byte = new_field_node.start_byte();
        // Create the builder and call the callback
        let mut builder = FieldAnnotationBuilder::new(ts_file, field_start_byte);
        Some(callback(&mut builder))
    } else {
        // Replacement failed
        None
    }
}
