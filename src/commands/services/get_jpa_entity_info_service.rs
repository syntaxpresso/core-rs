use std::path::Path;

use base64::prelude::*;
use tree_sitter::Node;

use crate::common::services::class_declaration_service::get_class_declaration_name_node;
use crate::common::services::package_declaration_service::{
    get_package_class_scope_node, get_package_declaration_node,
};
use crate::common::services::{
    annotation_service, class_declaration_service, field_declaration_service,
};
use crate::common::ts_file::TSFile;
use crate::responses::get_jpa_entity_info_response::{
    GetJpaEntityInfoResponse, IdFieldSearchResult,
};

fn decode_base64_to_bytes(b64: &str) -> Result<Vec<u8>, String> {
    BASE64_STANDARD
        .decode(b64)
        .map_err(|e| format!("Failed to decode base64: {}", e))
}

fn bytes_to_string(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_vec())
        .map_err(|e| format!("Failed to convert bytes to string: {}", e))
}

fn get_public_class_node<'a>(ts_file: &'a TSFile) -> Result<Node<'a>, String> {
    let public_class_node = class_declaration_service::get_public_class_node(ts_file);
    match public_class_node {
        Some(node) => Ok(node),
        None => Err("Unable to get public class node".to_string()),
    }
}

fn check_is_jpa_entity(ts_file: &TSFile, class_node: &Node) -> bool {
    annotation_service::find_annotation_node_by_name(ts_file, *class_node, "Entity").is_some()
}

fn extract_entity_type(ts_file: &TSFile, class_declaration_node: &Node) -> Result<String, String> {
    let class_name_node_option = get_class_declaration_name_node(ts_file, *class_declaration_node);
    if class_name_node_option.is_none() {
        return Err("Couldn't find class name node".to_string());
    }
    let class_name_node = class_name_node_option.unwrap();
    let class_name_option = ts_file.get_text_from_node(&class_name_node);
    if class_name_option.is_none() {
        return Err("Couldn't get the class name from the tree".to_string());
    }
    Ok(class_name_option.unwrap().to_string())
}

fn extract_entity_package_scope(ts_file: &TSFile) -> Result<String, String> {
    let package_declaration_node_option = get_package_declaration_node(ts_file);
    if package_declaration_node_option.is_none() {
        return Err("Unable to get package declaration node".to_string());
    }
    let package_declaration_node = package_declaration_node_option.unwrap();
    let package_scope_node_option = get_package_class_scope_node(ts_file, package_declaration_node);
    if package_scope_node_option.is_none() {
        return Err("Unable to get package scope".to_string());
    }
    let package_scope_node = package_scope_node_option.unwrap();
    let package_scope_option = ts_file.get_text_from_node(&package_scope_node);
    if package_scope_option.is_none() {
        return Err("Unable to get package scope".to_string());
    }
    let package_scope = package_scope_option.unwrap();
    Ok(package_scope.to_string())
}

fn find_id_field_info(
    ts_file: &TSFile,
    class_node: &Node,
) -> Result<(Option<String>, Option<String>), String> {
    let id_field_result = find_id_field_recursive(ts_file, class_node)?;
    match id_field_result {
        IdFieldSearchResult::Found {
            field_type,
            package_name,
        } => Ok((Some(field_type), Some(package_name))),
        _ => Ok((None, None)),
    }
}

fn find_id_field_recursive(
    ts_file: &TSFile,
    class_node: &Node,
) -> Result<IdFieldSearchResult, String> {
    let field_nodes =
        field_declaration_service::get_all_field_declaration_nodes(ts_file, *class_node);
    for field_node in field_nodes {
        if annotation_service::find_annotation_node_by_name(ts_file, field_node, "Id").is_some() {
            let field_type_result = field_declaration_service::get_field_declaration_full_type_node(
                ts_file, field_node,
            );
            if let Some(type_node) = field_type_result
                && let Some(field_type) = ts_file.get_text_from_node(&type_node)
            {
                let package_name = get_package_for_type(field_type);
                return Ok(IdFieldSearchResult::Found {
                    field_type: field_type.to_string(),
                    package_name,
                });
            }
        }
    }
    let superclass_name = extract_superclass_name(ts_file, class_node)?;
    if let Some(superclass) = superclass_name
        && is_external_class(&superclass)
    {
        return Ok(IdFieldSearchResult::ExternalSuperclass {
            superclass_name: superclass,
        });
    }
    Ok(IdFieldSearchResult::NotFound)
}

fn extract_superclass_name(ts_file: &TSFile, class_node: &Node) -> Result<Option<String>, String> {
    let mut cursor = class_node.walk();
    if !cursor.goto_first_child() {
        return Ok(None);
    }
    loop {
        let node = cursor.node();
        if node.kind() == "superclass"
            && let Some(type_node) = find_type_identifier_in_superclass(&node)
            && let Some(superclass_name) = ts_file.get_text_from_node(&type_node)
        {
            return Ok(Some(superclass_name.to_string()));
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    Ok(None)
}

fn find_type_identifier_in_superclass<'a>(superclass_node: &Node<'a>) -> Option<Node<'a>> {
    let mut cursor = superclass_node.walk();
    if !cursor.goto_first_child() {
        return None;
    }
    loop {
        let node = cursor.node();
        if node.kind() == "type_identifier" || node.kind() == "identifier" {
            return Some(node);
        }
        if node.kind() == "generic_type"
            && let Some(identifier) = find_first_identifier_in_node(&node)
        {
            return Some(identifier);
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    None
}

fn find_first_identifier_in_node<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    if !cursor.goto_first_child() {
        return None;
    }
    loop {
        let current_node = cursor.node();
        if current_node.kind() == "type_identifier" || current_node.kind() == "identifier" {
            return Some(current_node);
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    None
}

fn is_external_class(class_name: &str) -> bool {
    !matches!(
        class_name,
        "Object"
            | "String"
            | "Integer"
            | "Long"
            | "Double"
            | "Float"
            | "Boolean"
            | "BigDecimal"
            | "BigInteger"
    )
}

fn get_package_for_type(type_name: &str) -> String {
    match type_name {
        "String" => "java.lang".to_string(),
        "Integer" => "java.lang".to_string(),
        "Long" => "java.lang".to_string(),
        "Double" => "java.lang".to_string(),
        "Float" => "java.lang".to_string(),
        "Boolean" => "java.lang".to_string(),
        "BigDecimal" => "java.math".to_string(),
        "BigInteger" => "java.math".to_string(),
        "UUID" => "java.util".to_string(),
        "Date" => "java.util".to_string(),
        "LocalDate" => "java.time".to_string(),
        "LocalDateTime" => "java.time".to_string(),
        "Instant" => "java.time".to_string(),
        _ => String::new(),
    }
}

fn create_ts_file(
    entity_file_path: Option<&Path>,
    b64_superclass_source: Option<&str>,
) -> Result<TSFile, String> {
    if let Some(path) = entity_file_path {
        Ok(TSFile::from_file(path).map_err(|e| e.to_string())?)
    } else if let Some(b64) = b64_superclass_source {
        let bytes = decode_base64_to_bytes(b64)?;
        let source = bytes_to_string(&bytes)?;
        Ok(TSFile::from_source_code(&source))
    } else {
        Err("No source provided".to_string())
    }
}

pub fn run(
    entity_file_path: Option<&Path>,
    b64_source_code: Option<&str>,
) -> Result<GetJpaEntityInfoResponse, String> {
    // Step 1: Create TSFile
    let ts_file = create_ts_file(entity_file_path, b64_source_code)?;
    // Step 2: Get public class node
    let public_class_node = get_public_class_node(&ts_file)?;
    // Step 3: Check if class is JPA entity
    let is_jpa_entity = check_is_jpa_entity(&ts_file, &public_class_node);
    // Step 4: Extract class name
    let entity_type = extract_entity_type(&ts_file, &public_class_node)?;
    // Step 5: Extract package name
    let entity_package_name = extract_entity_package_scope(&ts_file)?;
    // Step 6: Find ID field info
    let (id_field_type, id_field_package_name) = find_id_field_info(&ts_file, &public_class_node)?;
    // Step 7: Build and return response
    let entity_path = ts_file
        .file_path()
        .map(|path| path.to_string_lossy().to_string());
    Ok(GetJpaEntityInfoResponse {
        is_jpa_entity,
        entity_type,
        entity_package_name,
        entity_path,
        id_field_type,
        id_field_package_name,
    })
}
