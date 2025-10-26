#![allow(dead_code)]

use crate::common::ts_file::TSFile;
use std::collections::HashMap;
use tree_sitter::Node;

fn get_first_public_class_node<'a>(ts_file: &'a TSFile) -> Option<Node<'a>> {
    ts_file.tree.as_ref()?;
    let query_string = r#"
        (class_declaration
          (modifiers) @modifiers
          name: (identifier) @className
        ) @classDeclaration
    "#;
    if let Ok(results) = ts_file
        .query_builder(query_string)
        .returning_all_captures()
        .execute()
    {
        let captures = results.captures();
        for capture_map in captures {
            if let Some(modifiers_node) = capture_map.get("modifiers")
                && let Some(modifier_text) = ts_file.get_text_from_node(modifiers_node)
                && modifier_text.contains("public")
            {
                return capture_map.get("classDeclaration").copied();
            }
        }
    }
    None
}

pub fn find_class_node_by_name<'a>(ts_file: &'a TSFile, class_name: &str) -> Option<Node<'a>> {
    ts_file.tree.as_ref()?;
    let query_string = format!(
        r#"
        (class_declaration
            name: (identifier) @className
        (#eq? @className "{}")) @classDeclaration
        "#,
        class_name
    );
    ts_file
        .query_builder(&query_string)
        .returning("classDeclaration")
        .execute()
        .ok()?
        .first_node()
}

pub fn get_public_class_node<'a>(ts_file: &'a TSFile) -> Option<Node<'a>> {
    ts_file.tree.as_ref()?;
    match ts_file.get_file_name_without_ext() {
        Some(ref file_name) if !file_name.is_empty() => find_class_node_by_name(ts_file, file_name),
        _ => get_first_public_class_node(ts_file),
    }
}

pub fn get_all_class_declaration_nodes<'a>(ts_file: &'a TSFile) -> Vec<HashMap<String, Node<'a>>> {
    if ts_file.tree.is_none() {
        return Vec::new();
    }
    let query_string = r#"
        (class_declaration
          name: (identifier) @className) @classDeclaration
    "#;
    match ts_file
        .query_builder(query_string)
        .returning_all_captures()
        .execute()
    {
        Ok(result) => result
            .captures()
            .iter()
            .map(|m| m.captures.clone())
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub fn get_class_declaration_name_node<'a>(
    ts_file: &'a TSFile,
    class_node: Node<'a>,
) -> Option<Node<'a>> {
    if class_node.kind() != "class_declaration" {
        return None;
    }
    let query_string = r#"
        name: (identifier) @className
    "#;
    ts_file
        .query_builder(query_string)
        .within(class_node)
        .returning("className")
        .execute()
        .ok()?
        .first_node()
}

pub fn get_class_superclass_name_node<'a>(
    ts_file: &'a TSFile,
    class_declaration_node: Node<'a>,
) -> Option<Node<'a>> {
    ts_file.tree.as_ref()?;
    let query_string = r#"
        (class_declaration
          name: (identifier) @className
          (superclass
            (type_identifier) @superclassName
          )?
        )
    "#;
    ts_file
        .query_builder(query_string)
        .within(class_declaration_node)
        .returning("superclassName")
        .execute()
        .ok()?
        .first_node()
}
