#![allow(dead_code)]

use crate::common::ts_file::TSFile;
use tree_sitter::Node;

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
