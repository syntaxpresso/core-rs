#![allow(dead_code)]

use crate::common::ts_file::TSFile;
use crate::common::types::annotation_types::AnnotationInsertionPosition;
use tree_sitter::Node;

#[derive(Debug, Clone)]
struct AnnotationInsertionPoint {
    position: AnnotationInsertionPosition,
    insert_byte: usize,
    break_line_before: bool,
    break_line_after: bool,
}

impl Default for AnnotationInsertionPoint {
    fn default() -> Self {
        Self::new()
    }
}

impl AnnotationInsertionPoint {
    fn new() -> Self {
        Self {
            position: AnnotationInsertionPosition::BeforeFirstAnnotation,
            insert_byte: 0,
            break_line_before: false,
            break_line_after: false,
        }
    }
}

pub fn get_all_annotation_nodes<'a>(ts_file: &'a TSFile, scope_node: Node<'a>) -> Vec<Node<'a>> {
    if ts_file.tree.is_none() {
        return Vec::new();
    }
    let query_string = r#"
        (
          [
            (annotation) @annotation
            (marker_annotation) @markerAnnotation
          ]
        )
    "#;
    match ts_file
        .query_builder(query_string)
        .within(scope_node)
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

pub fn get_annotation_name_node<'a>(ts_file: &'a TSFile, scope_node: Node<'a>) -> Option<Node<'a>> {
    ts_file.tree.as_ref()?;
    let query_string = r#"
        [
          (annotation
            name: (identifier) @annotationName
          )
          (marker_annotation
            name: (identifier) @annotationName
          )
        ]
    "#;
    ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("annotationName")
        .execute()
        .ok()?
        .first_node()
}

pub fn find_annotation_node_by_name<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
    annotation_name: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || annotation_name.trim().is_empty() {
        return None;
    }
    let query_string = format!(
        r#"
        (
          [
            (annotation name: (identifier) @name)
            (marker_annotation name: (identifier) @name)
          ] @node
          (#eq? @name "{}")
        )
        "#,
        annotation_name
    );
    ts_file
        .query_builder(&query_string)
        .within(scope_node)
        .returning("node")
        .execute()
        .ok()?
        .first_node()
}

pub fn get_annotation_argument_pair_nodes<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Vec<Node<'a>> {
    if ts_file.tree.is_none() {
        return Vec::new();
    }
    let query_string = r#"
        (
          (annotation_argument_list
            (element_value_pair) @pair
          )
        )
    "#;
    match ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("pair")
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

pub fn get_annotation_argument_key_nodes<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Vec<Node<'a>> {
    if ts_file.tree.is_none() {
        return Vec::new();
    }
    let query_string = r#"
        (
          (annotation_argument_list
            (element_value_pair
              key: (_) @key
            )
          )
        )
    "#;
    match ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("key")
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

pub fn get_annotation_argument_value_nodes<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
) -> Vec<Node<'a>> {
    if ts_file.tree.is_none() {
        return Vec::new();
    }
    let query_string = r#"
        (
          (annotation_argument_list
            (element_value_pair
              value: (_) @value
            )
          )
        )
    "#;
    match ts_file
        .query_builder(query_string)
        .within(scope_node)
        .returning("value")
        .execute()
    {
        Ok(result) => result.nodes(),
        Err(_) => Vec::new(),
    }
}

pub fn find_annotation_value_node_by_key<'a>(
    ts_file: &'a TSFile,
    scope_node: Node<'a>,
    key: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || key.trim().is_empty() {
        return None;
    }
    let query_string = format!(
        r#"
        (
          (annotation_argument_list
            (element_value_pair
              key: (identifier) @key
              value: (_) @value
            )
          )
          (#eq? @key "{}")
        )
        "#,
        key
    );
    ts_file
        .query_builder(&query_string)
        .within(scope_node)
        .returning("value")
        .execute()
        .ok()?
        .first_node()
}

pub fn add_annotation<'a>(
    ts_file: &'a mut TSFile,
    declaration_byte_position: usize,
    insertion_position: &AnnotationInsertionPosition,
    annotation_text: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || annotation_text.trim().is_empty() {
        return None;
    }
    // Collect all necessary information before any mutable operations
    let (declaration_start_byte, declaration_end_byte, _node_kind, current_text, all_annotations) = {
        // Find the declaration node at the given byte position
        let mut declaration_node =
            ts_file.get_named_node_at_byte_position(declaration_byte_position);
        declaration_node.as_ref()?;
        let mut current_node = declaration_node.unwrap();
        let mut node_kind = current_node.kind();
        // If we found a modifiers node, look for the parent class_declaration
        if node_kind == "modifiers"
            && let Some(parent) = current_node.parent()
            && parent.kind() == "class_declaration"
        {
            current_node = parent;
            node_kind = current_node.kind();
        }
        // If we found an annotation, navigate up to find the containing class_declaration
        if matches!(node_kind, "annotation" | "marker_annotation") {
            // The annotation is likely a child of modifiers, which is a child of class_declaration
            // So we need to go up the parent chain to find class_declaration
            let mut current_ancestor = Some(current_node);
            while let Some(ancestor) = current_ancestor {
                if ancestor.kind() == "class_declaration" {
                    current_node = ancestor;
                    node_kind = current_node.kind();
                    break;
                }
                current_ancestor = ancestor.parent();
            }
        }
        if !matches!(
            node_kind,
            "class_declaration"
                | "field_declaration"
                | "interface_declaration"
                | "method_declaration"
        ) {
            return None;
        }
        declaration_node = Some(current_node);
        let declaration_node = declaration_node.unwrap();
        let all_annotations = get_all_annotation_nodes(ts_file, declaration_node);
        let current_text = ts_file.get_text_from_node(&declaration_node);
        current_text.as_ref()?;
        let current_text = current_text.unwrap().to_string();
        (
            declaration_node.start_byte(),
            declaration_node.end_byte(),
            node_kind,
            current_text,
            all_annotations,
        )
    };
    let mut annotation_insertion_point = AnnotationInsertionPoint::new();
    annotation_insertion_point.position = insertion_position.clone();
    match insertion_position {
        AnnotationInsertionPosition::BeforeFirstAnnotation => {
            if !all_annotations.is_empty() {
                annotation_insertion_point.break_line_after = true;
                annotation_insertion_point.insert_byte = all_annotations[0].start_byte();
            } else {
                annotation_insertion_point.break_line_after = true;
                annotation_insertion_point.insert_byte = declaration_start_byte;
            }
        }
        AnnotationInsertionPosition::AboveScopeDeclaration => {
            if all_annotations.is_empty() {
                annotation_insertion_point.break_line_after = true;
                annotation_insertion_point.insert_byte = declaration_start_byte;
            } else {
                annotation_insertion_point.break_line_before = true;
                annotation_insertion_point.insert_byte = all_annotations.last()?.end_byte();
            }
        }
    }
    let new_content = match insertion_position {
        AnnotationInsertionPosition::BeforeFirstAnnotation => {
            if !all_annotations.is_empty() {
                // Insert before first annotation
                let first_annotation = &all_annotations[0];
                let relative_pos = first_annotation.start_byte() - declaration_start_byte;
                let before = &current_text[..relative_pos];
                let after = &current_text[relative_pos..];
                format!("{}{}\n{}", before, annotation_text, after)
            } else {
                // No annotations exist, insert at beginning
                format!("{}\n{}", annotation_text, current_text)
            }
        }
        AnnotationInsertionPosition::AboveScopeDeclaration => {
            if all_annotations.is_empty() {
                // No annotations exist, insert at beginning
                format!("{}\n{}", annotation_text, current_text)
            } else {
                // Insert after last annotation
                let last_annotation = all_annotations.last()?;
                let relative_pos = last_annotation.end_byte() - declaration_start_byte;
                let before = &current_text[..relative_pos];
                let after = &current_text[relative_pos..];
                format!("{}\n{}{}", before, annotation_text, after)
            }
        }
    };
    ts_file.replace_text_by_byte_range(declaration_start_byte, declaration_end_byte, &new_content)
}

pub fn add_annotation_argument<'a>(
    ts_file: &'a mut TSFile,
    annotation_byte_position: usize,
    key: &str,
    value: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || key.trim().is_empty() || value.trim().is_empty() {
        return None;
    }
    // Collect all necessary information before any mutable operations
    let (
        annotation_start_byte,
        annotation_end_byte,
        node_kind,
        current_text,
        name_node_info,
        existing_arguments,
    ) = {
        // Find the annotation node at the given byte position
        let annotation_node = ts_file.get_named_node_at_byte_position(annotation_byte_position)?;
        let node_kind = annotation_node.kind();
        if !matches!(node_kind, "annotation" | "marker_annotation") {
            return None;
        }
        let current_text = ts_file.get_text_from_node(&annotation_node)?.to_string();
        let name_node_info = get_annotation_name_node(ts_file, annotation_node)
            .map(|n| n.end_byte() - annotation_node.start_byte());
        let existing_arguments = get_annotation_argument_pair_nodes(ts_file, annotation_node);
        (
            annotation_node.start_byte(),
            annotation_node.end_byte(),
            node_kind,
            current_text,
            name_node_info,
            existing_arguments,
        )
    };
    let argument_pair = format!("{} = {}", key, value);
    let new_content = if node_kind == "marker_annotation" {
        // Convert marker annotation to annotation with arguments
        // @Test -> @Test(key = value)
        let name_end_pos = name_node_info?;
        let before = &current_text[..name_end_pos];
        let after = &current_text[name_end_pos..];
        format!("{}({}){}", before, argument_pair, after)
    } else {
        // Add argument to existing annotation
        if existing_arguments.is_empty() {
            // No existing arguments, add first one
            // @Column -> @Column(key = value)
            let name_end_pos = name_node_info?;
            let before = &current_text[..name_end_pos];
            let after = &current_text[name_end_pos..];
            format!("{}({}){}", before, argument_pair, after)
        } else {
            // Add argument to existing arguments
            // @Column(name = "test") -> @Column(name = "test", key = value)
            let last_argument = existing_arguments.last()?;
            let insert_pos = last_argument.end_byte() - annotation_start_byte;
            let before = &current_text[..insert_pos];
            let after = &current_text[insert_pos..];
            format!("{}, {}{}", before, argument_pair, after)
        }
    };
    ts_file.replace_text_by_byte_range(annotation_start_byte, annotation_end_byte, &new_content)
}
