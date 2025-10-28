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
    // Add the field and annotations using the callback pattern
    add_field_declaration(
        &mut entity_ts_file,
        public_class_node_start_byte,
        params,
        |builder| {
            builder
                .add_annotation("@Column")?
                .with_argument("@Column", "name", "\"test\"")?
                .add_annotation("@JsonView")?
                .with_value("@JsonView", "Views.Public.class")?
                .build()
        },
    )
    .ok_or_else(|| "Unable to add new field to the JPA Entity".to_string())?
    .map_err(|e| format!("Unable to add annotations: {}", e))?;
    entity_ts_file
        .save()
        .map_err(|e| format!("Unable to save JPA Entity file: {}", e))?;
    Ok(FileResponse {
        file_type: "oi".to_string(),
        file_package_name: "oi".to_string(),
        file_path: "oi".to_string(),
    })
}
