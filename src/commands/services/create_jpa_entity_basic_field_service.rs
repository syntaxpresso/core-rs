use crate::common::services::class_declaration_service::get_public_class_node;
use crate::common::services::field_declaration_service::{
    AddFieldDeclarationParams, add_field_declaration,
};
use crate::common::ts_file::TSFile;
use crate::common::types::basic_field_config::BasicFieldConfig;
use crate::common::types::field_types::FieldInsertionPosition;
use crate::common::types::java_visibility_modifier::JavaVisibilityModifier;
use crate::responses::file_response::FileResponse;
use std::path::Path;

pub fn run(
    _cwd: &Path,
    entity_file_path: &Path,
    field_config: BasicFieldConfig,
) -> Result<FileResponse, String> {
    let mut entity_ts_file = TSFile::from_file(entity_file_path)
        .map_err(|_| "Unable to parse JPA Entity file".to_string())?;
    let public_class_node = get_public_class_node(&entity_ts_file)
        .ok_or_else(|| "Unable to get public class node from Entity".to_string())?;
    let public_class_node_start_byte = public_class_node.start_byte();
    let params = AddFieldDeclarationParams {
        insertion_position: FieldInsertionPosition::EndOfClassBody,
        visibility_modifier: JavaVisibilityModifier::Private,
        field_modifiers: vec![],
        field_type: &field_config.field_type,
        field_name: &field_config.field_name,
        field_initialization: None,
    };
    // Add the field and get a confirmation that it was added

    let new_field_pos =
        add_field_declaration(&mut entity_ts_file, public_class_node_start_byte, params)
            .ok_or_else(|| "Unable to add new field to the JPA Entity".to_string())?;

    let new_field = entity_ts_file
        .get_node_at_byte_position_with_kind(new_field_pos, "field_declaration")
        .ok_or_else(|| "Unable to find the newly added field".to_string())?;

    let new_field_str = entity_ts_file
        .get_text_from_node(&new_field)
        .ok_or_else(|| "".to_string())?;

    println!("{}", new_field_str);

    entity_ts_file
        .save()
        .map_err(|e| format!("Unable to save JPA Entity file: {}", e))?;
    Ok(FileResponse {
        file_type: "oi".to_string(),
        file_package_name: "oi".to_string(),
        file_path: "oi".to_string(),
    })
}
