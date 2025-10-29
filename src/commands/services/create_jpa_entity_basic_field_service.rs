use crate::common::services::class_declaration_service::get_public_class_node;
use crate::common::services::field_declaration_service::{
    AddFieldDeclarationParams, add_field_declaration,
};
use crate::common::services::import_declaration_service::add_import;
use crate::common::ts_file::TSFile;
use crate::common::types::basic_field_config::BasicFieldConfig;
use crate::common::types::field_types::FieldInsertionPosition;
use crate::common::types::import_types::ImportInsertionPosition;
use crate::common::types::java_field_temporal::JavaFieldTemporal;
use crate::common::types::java_field_time_zone_storage::JavaFieldTimeZoneStorage;
use crate::common::types::java_visibility_modifier::JavaVisibilityModifier;
use crate::common::utils::case_util::{self, CaseType};
use crate::responses::file_response::FileResponse;
use std::collections::{HashMap, HashSet};
use std::path::Path;

struct ProcessedFieldConfig {
    pub should_add_timezone_storage_annotation: bool,
    pub should_add_temporal_annotation: bool,
    pub should_add_lob_annotation: bool,
}

fn add_to_import_map(hashmap: &mut HashMap<String, String>, class_name: &str, package_name: &str) {
    if !hashmap.contains_key(class_name) {
        hashmap.insert(class_name.to_string(), package_name.to_string());
    }
}

fn add_imports(ts_file: &mut TSFile, import_map: &HashMap<String, String>) {
    let import_position = ImportInsertionPosition::AfterLastImport;
    for (package_name, class_name) in import_map {
        add_import(ts_file, &import_position, package_name, class_name);
    }
}

fn process_imports(
    import_map: &mut HashMap<String, String>,
    processed_field_config: &ProcessedFieldConfig,
) {
    add_to_import_map(import_map, "jakarta.persistence", "Column");
    if processed_field_config.should_add_timezone_storage_annotation {
        add_to_import_map(import_map, "org.hibernate.annotations", "TimeZoneStorage");
        add_to_import_map(
            import_map,
            "org.hibernate.annotations",
            "TimeZoneStorageType",
        );
        add_to_import_map(import_map, "java.time", "OffsetDateTime");
    }
    if processed_field_config.should_add_temporal_annotation {
        add_to_import_map(import_map, "jakarta.persistence", "Temporal");
        add_to_import_map(import_map, "jakarta.persistence", "TemporalType");
        add_to_import_map(import_map, "java.sql", "Date");
    }
    if processed_field_config.should_add_lob_annotation {}
}

fn process_field_config(field_config: &BasicFieldConfig) -> ProcessedFieldConfig {
    let mut should_add_timezone_storage_annotation = false;
    let mut should_add_temporal_annotation = false;
    let mut should_add_lob_annotation = false;
    let time_zone_aware_types: HashSet<&str> = [
        "java.time.OffsetDateTime",
        "java.time.ZonedDateTime",
        "java.time.OffsetTime",
    ]
    .iter()
    .cloned()
    .collect();
    let temporal_aware_types: HashSet<&str> =
        ["java.util.Date", "java.util.Calendar", "java.sql.Date"]
            .iter()
            .cloned()
            .collect();
    let lob_aware_types: HashSet<&str> = [
        "java.lang.String",
        "byte[]",
        "java.lang.Byte[]",
        "char[]",
        "java.lang.Character[]",
    ]
    .iter()
    .cloned()
    .collect();
    let full_type = field_config
        .field_type_package_name
        .as_ref()
        .map(|pkg| format!("{}.{}", pkg, field_config.field_type))
        .unwrap_or_else(|| field_config.field_type.clone());
    if time_zone_aware_types.contains(full_type.as_str()) {
        should_add_timezone_storage_annotation = true;
    }
    if temporal_aware_types.contains(full_type.as_str()) && field_config.field_temporal.is_some() {
        should_add_temporal_annotation = true;
    }
    if lob_aware_types.contains(full_type.as_str())
        && lob_aware_types.contains(field_config.field_type.as_str())
    {
        should_add_lob_annotation = true;
    }
    ProcessedFieldConfig {
        should_add_timezone_storage_annotation,
        should_add_temporal_annotation,
        should_add_lob_annotation,
    }
}

fn add_field_and_annotations(
    ts_file: &mut TSFile,
    field_config: &BasicFieldConfig,
    processed_field_config: &ProcessedFieldConfig,
) -> Result<(), String> {
    let public_class_node = get_public_class_node(ts_file)
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
    let column_name_snake_case =
        case_util::auto_convert_case(&field_config.field_name, CaseType::Snake);
    let timezone_storage_type = field_config
        .field_timezone_storage
        .clone()
        .unwrap_or(JavaFieldTimeZoneStorage::Auto);
    let temporal_type = field_config
        .field_temporal
        .clone()
        .unwrap_or(JavaFieldTemporal::Timestamp);
    add_field_declaration(ts_file, public_class_node_start_byte, params, |builder| {
        builder.add_annotation("@Column")?.with_argument(
            "@Column",
            "name",
            &format!("\"{}\"", &column_name_snake_case),
        )?;
        if processed_field_config.should_add_timezone_storage_annotation
            && timezone_storage_type.ne(&JavaFieldTimeZoneStorage::Auto)
        {
            builder.add_annotation("@TimeZoneStorage")?.with_argument(
                "@TimeZoneStorage",
                "value",
                &format!("TimeZoneStorageType.{}", timezone_storage_type.as_str()),
            )?;
        }
        if processed_field_config.should_add_temporal_annotation {
            builder.add_annotation("@Temporal")?.with_argument(
                "@Temporal",
                "value",
                &format!("TemporalType.{}", temporal_type.as_str()),
            )?;
        }
        if processed_field_config.should_add_lob_annotation {
            builder.add_annotation("@Lob")?;
        }
        builder.build()
    })
    .ok_or_else(|| "Unable to add new field to the JPA Entity".to_string())?
    .map_err(|e| format!("Unable to add annotations: {}", e))?;
    Ok(())
}

pub fn run(
    _cwd: &Path,
    entity_file_path: &Path,
    field_config: BasicFieldConfig,
) -> Result<FileResponse, String> {
    let mut import_map: HashMap<String, String> = HashMap::new();
    let processed_field_config = process_field_config(&field_config);
    let mut entity_ts_file = TSFile::from_file(entity_file_path)
        .map_err(|_| "Unable to parse JPA Entity file".to_string())?;
    process_imports(&mut import_map, &processed_field_config);
    add_field_and_annotations(&mut entity_ts_file, &field_config, &processed_field_config)?;
    add_imports(&mut entity_ts_file, &import_map);
    entity_ts_file
        .save()
        .map_err(|e| format!("Unable to save JPA Entity file: {}", e))?;
    Ok(FileResponse {
        file_type: "oi".to_string(),
        file_package_name: "oi".to_string(),
        file_path: "oi".to_string(),
    })
}
