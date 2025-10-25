#![allow(dead_code)]

use crate::common::ts_file::TSFile;
use tree_sitter::Node;

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
