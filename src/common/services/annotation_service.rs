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
    declaration_node: Node<'a>,
    insertion_position: &AnnotationInsertionPosition,
    annotation_text: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || annotation_text.trim().is_empty() {
        return None;
    }
    let node_kind = declaration_node.kind();
    if !matches!(
        node_kind,
        "class_declaration" | "field_declaration" | "interface_declaration" | "method_declaration"
    ) {
        return None;
    }
    let all_annotations = get_all_annotation_nodes(ts_file, declaration_node);
    let mut annotation_insertion_point = AnnotationInsertionPoint::new();
    annotation_insertion_point.position = insertion_position.clone();
    match insertion_position {
        AnnotationInsertionPosition::BeforeFirstAnnotation => {
            if !all_annotations.is_empty() {
                annotation_insertion_point.break_line_after = true;
                annotation_insertion_point.insert_byte = all_annotations[0].start_byte();
            } else {
                annotation_insertion_point.break_line_after = true;
                annotation_insertion_point.insert_byte = declaration_node.start_byte();
            }
        }
        AnnotationInsertionPosition::AboveScopeDeclaration => {
            if all_annotations.is_empty() {
                annotation_insertion_point.break_line_after = true;
                annotation_insertion_point.insert_byte = declaration_node.start_byte();
            } else {
                annotation_insertion_point.break_line_before = true;
                annotation_insertion_point.insert_byte = all_annotations.last()?.end_byte();
            }
        }
    }
    let current_text = ts_file.get_text_from_node(&declaration_node)?;
    let new_content = match insertion_position {
        AnnotationInsertionPosition::BeforeFirstAnnotation => {
            if !all_annotations.is_empty() {
                // Insert before first annotation
                let first_annotation = &all_annotations[0];
                let relative_pos = first_annotation.start_byte() - declaration_node.start_byte();
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
                let relative_pos = last_annotation.end_byte() - declaration_node.start_byte();
                let before = &current_text[..relative_pos];
                let after = &current_text[relative_pos..];
                format!("{}\n{}{}", before, annotation_text, after)
            }
        }
    };
    ts_file.replace_text_by_node(&declaration_node, &new_content)
}

pub fn add_annotation_argument<'a>(
    ts_file: &'a mut TSFile,
    annotation_node: Node<'a>,
    key: &str,
    value: &str,
) -> Option<Node<'a>> {
    if ts_file.tree.is_none() || key.trim().is_empty() || value.trim().is_empty() {
        return None;
    }
    let node_kind = annotation_node.kind();
    if !matches!(node_kind, "annotation" | "marker_annotation") {
        return None;
    }
    let current_text = ts_file.get_text_from_node(&annotation_node)?;
    let argument_pair = format!("{} = {}", key, value);
    let new_content = if node_kind == "marker_annotation" {
        // Convert marker annotation to annotation with arguments
        // @Test -> @Test(key = value)
        let name_node = get_annotation_name_node(ts_file, annotation_node)?;
        let name_end_pos = name_node.end_byte() - annotation_node.start_byte();
        let before = &current_text[..name_end_pos];
        let after = &current_text[name_end_pos..];
        format!("{}({}){}", before, argument_pair, after)
    } else {
        // Add argument to existing annotation
        let existing_arguments = get_annotation_argument_pair_nodes(ts_file, annotation_node);
        if existing_arguments.is_empty() {
            // No existing arguments, add first one
            // @Column -> @Column(key = value)
            let name_node = get_annotation_name_node(ts_file, annotation_node)?;
            let name_end_pos = name_node.end_byte() - annotation_node.start_byte();
            let before = &current_text[..name_end_pos];
            let after = &current_text[name_end_pos..];
            format!("{}({}){}", before, argument_pair, after)
        } else {
            // Add argument to existing arguments
            // @Column(name = "test") -> @Column(name = "test", key = value)
            let last_argument = existing_arguments.last()?;
            let insert_pos = last_argument.end_byte() - annotation_node.start_byte();
            let before = &current_text[..insert_pos];
            let after = &current_text[insert_pos..];
            format!("{}, {}{}", before, argument_pair, after)
        }
    };
    ts_file.replace_text_by_node(&annotation_node, &new_content)
}
