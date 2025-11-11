#![allow(dead_code)]

use crate::common::ts_file::TSFile;
use tree_sitter::Node;

pub fn find_record_node_by_name<'a>(ts_file: &'a TSFile, record_name: &str) -> Option<Node<'a>> {
  ts_file.tree.as_ref()?;
  let query_string = format!(
    r#"
        (record_declaration
          name: (identifier) @recordName
        (#eq? @recordName "{}")) @recordDeclaration
        "#,
    record_name
  );
  ts_file.query_builder(&query_string).returning("recordDeclaration").execute().ok()?.first_node()
}

pub fn get_first_public_record_node<'a>(ts_file: &'a TSFile) -> Option<Node<'a>> {
  ts_file.tree.as_ref()?;
  let query_string = r#"
        (record_declaration
          (modifiers) @modifiers
          name: (identifier) @recordName
        ) @recordDeclaration
    "#;
  if let Ok(results) = ts_file.query_builder(query_string).returning_all_captures().execute() {
    let captures = results.captures();
    for capture_map in captures {
      if let Some(modifiers_node) = capture_map.get("modifiers")
        && let Some(modifier_text) = ts_file.get_text_from_node(modifiers_node)
        && modifier_text.contains("public")
      {
        return capture_map.get("recordDeclaration").copied();
      }
    }
  }
  None
}

pub fn get_public_record_node<'a>(ts_file: &'a TSFile) -> Option<Node<'a>> {
  ts_file.tree.as_ref()?;
  match ts_file.get_file_name_without_ext() {
    Some(ref file_name) if !file_name.is_empty() => find_record_node_by_name(ts_file, file_name),
    _ => get_first_public_record_node(ts_file),
  }
}

pub fn get_record_name_node<'a>(
  ts_file: &'a TSFile,
  record_declaration_node: Node<'a>,
) -> Option<Node<'a>> {
  if ts_file.tree.is_none() || record_declaration_node.kind() != "record_declaration" {
    return None;
  }
  let query_string = r#"
        (record_declaration
          name: (identifier) @recordName
        ) @recordDeclaration
    "#;
  if let Ok(results) = ts_file
    .query_builder(query_string)
    .within(record_declaration_node)
    .returning_all_captures()
    .execute()
  {
    let captures = results.captures();
    for capture_map in captures {
      if let Some(name_node) = capture_map.get("recordName") {
        return Some(*name_node);
      }
    }
  }
  None
}
