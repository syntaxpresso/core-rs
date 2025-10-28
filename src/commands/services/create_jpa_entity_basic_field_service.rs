use crate::common::services::{annotation_service, field_declaration_service, import_declaration_service, class_declaration_service};
use crate::common::ts_file::TSFile;
use crate::common::types::annotation_types::AnnotationInsertionPosition;
use crate::common::types::basic_field_config::BasicFieldConfig;
use crate::common::types::field_types::FieldInsertionPosition;
use crate::common::types::import_types::ImportInsertionPosition;
use crate::responses::file_response::FileResponse;
use std::path::Path;

fn load_ts_file(file_path: &Path) -> Result<TSFile, String> {
    TSFile::from_file(file_path)
        .map_err(|e| format!("Failed to load TSFile from {:?}: {}", file_path, e))
}

fn get_class_node(ts_file: &TSFile) -> Result<tree_sitter::Node<'_>, String> {
    class_declaration_service::get_public_class_node(ts_file)
        .ok_or("No public class found in entity file".to_string())
}

fn add_jpa_column_import(ts_file: &mut TSFile) -> Result<(), String> {
    import_declaration_service::add_import(
        ts_file,
        &ImportInsertionPosition::AfterLastImport,
        "jakarta.persistence",
        "Column",
    )
    .map(|_| ())
    .ok_or("Failed to add import for jakarta.persistence.Column".to_string())
}

fn add_temporal_imports(ts_file: &mut TSFile) -> Result<(), String> {
    import_declaration_service::add_import(
        ts_file,
        &ImportInsertionPosition::AfterLastImport,
        "jakarta.persistence",
        "Temporal",
    )
    .ok_or("Failed to add import for jakarta.persistence.Temporal")?;
    
    import_declaration_service::add_import(
        ts_file,
        &ImportInsertionPosition::AfterLastImport,
        "jakarta.persistence",
        "TemporalType",
    )
    .map(|_| ())
    .ok_or("Failed to add import for jakarta.persistence.TemporalType".to_string())
}

fn add_timezone_storage_imports(ts_file: &mut TSFile) -> Result<(), String> {
    import_declaration_service::add_import(
        ts_file,
        &ImportInsertionPosition::AfterLastImport,
        "org.hibernate.annotations",
        "TimeZoneStorage",
    )
    .ok_or("Failed to add import for org.hibernate.annotations.TimeZoneStorage")?;
    
    import_declaration_service::add_import(
        ts_file,
        &ImportInsertionPosition::AfterLastImport,
        "org.hibernate.annotations",
        "TimeZoneStorageType",
    )
    .map(|_| ())
    .ok_or("Failed to add import for org.hibernate.annotations.TimeZoneStorageType".to_string())
}

fn add_lob_import(ts_file: &mut TSFile) -> Result<(), String> {
    import_declaration_service::add_import(
        ts_file,
        &ImportInsertionPosition::AfterLastImport,
        "jakarta.persistence",
        "Lob",
    )
    .map(|_| ())
    .ok_or("Failed to add import for jakarta.persistence.Lob".to_string())
}

fn add_custom_type_import(ts_file: &mut TSFile, package_name: &str, type_name: &str) -> Result<(), String> {
    import_declaration_service::add_import(
        ts_file,
        &ImportInsertionPosition::AfterLastImport,
        package_name,
        type_name,
    )
    .map(|_| ())
    .ok_or(format!("Failed to add import for {}.{}", package_name, type_name))
}

fn generate_field_declaration(field_config: &BasicFieldConfig) -> String {
    format!("private {} {};", field_config.field_type, field_config.field_name)
}

fn generate_column_annotation(field_config: &BasicFieldConfig) -> String {
    let mut parts = Vec::new();
    
    // Add name (field name converted to snake_case is handled by the column annotation)
    parts.push(format!("name = \"{}\"", field_config.field_name));
    
    // Add length if specified
    if let Some(length) = field_config.field_length {
        parts.push(format!("length = {}", length));
    }
    
    // Add precision if specified
    if let Some(precision) = field_config.field_precision {
        parts.push(format!("precision = {}", precision));
    }
    
    // Add scale if specified and greater than 0
    if let Some(scale) = field_config.field_scale {
        if scale > 0 {
            parts.push(format!("scale = {}", scale));
        }
    }
    
    // Add nullable if explicitly set (default in JPA is nullable = true)
    if let Some(nullable) = field_config.field_nullable {
        if !nullable {
            parts.push("nullable = false".to_string());
        }
    }
    
    // Add unique if set to true
    if let Some(unique) = field_config.field_unique {
        if unique {
            parts.push("unique = true".to_string());
        }
    }
    
    if parts.is_empty() {
        "@Column".to_string()
    } else {
        format!("@Column({})", parts.join(", "))
    }
}

fn generate_temporal_annotation(field_config: &BasicFieldConfig) -> Option<String> {
    field_config.field_temporal.as_ref().map(|temporal| {
        format!("@Temporal(TemporalType.{})", temporal.as_str().to_uppercase())
    })
}

fn generate_timezone_storage_annotation(field_config: &BasicFieldConfig) -> Option<String> {
    field_config.field_timezone_storage.as_ref().map(|storage| {
        format!("@TimeZoneStorage(TimeZoneStorageType.{})", storage.as_str().to_uppercase())
    })
}

fn generate_lob_annotation(field_config: &BasicFieldConfig) -> Option<String> {
    field_config.field_large_object.and_then(|is_lob| {
        if is_lob {
            Some("@Lob".to_string())
        } else {
            None
        }
    })
}

fn add_field_with_annotations(
    ts_file: &mut TSFile,
    class_byte_position: usize,
    field_config: &BasicFieldConfig,
) -> Result<(), String> {
    // Generate the field declaration
    let field_declaration = generate_field_declaration(field_config);
    
    // Add the field declaration to the class
    field_declaration_service::add_field_declaration(
        ts_file,
        class_byte_position,
        &FieldInsertionPosition::AfterLastField,
        &field_declaration,
    )
    .ok_or("Failed to add field declaration to class")?;
    
    // After adding the field, we need to find it again to add annotations
    // Get updated class node and find the newly added field
    let updated_class_node = get_class_node(ts_file)?;
    let field_node = field_declaration_service::find_field_declaration_node_by_name(
        ts_file,
        &field_config.field_name,
        updated_class_node,
    )
    .ok_or("Failed to find newly added field for annotation")?;
    
    let field_byte_position = field_node.start_byte();
    
    // Add @Column annotation
    let column_annotation = generate_column_annotation(field_config);
    annotation_service::add_annotation(
        ts_file,
        field_byte_position,
        &AnnotationInsertionPosition::AboveScopeDeclaration,
        &column_annotation,
    )
    .ok_or("Failed to add @Column annotation to field")?;
    
    // Add @Temporal annotation if needed
    if let Some(temporal_annotation) = generate_temporal_annotation(field_config) {
        // Get updated field node position after previous annotation
        let updated_class_node = get_class_node(ts_file)?;
        let updated_field_node = field_declaration_service::find_field_declaration_node_by_name(
            ts_file,
            &field_config.field_name,
            updated_class_node,
        )
        .ok_or("Failed to find field for temporal annotation")?;
        
        annotation_service::add_annotation(
            ts_file,
            updated_field_node.start_byte(),
            &AnnotationInsertionPosition::AboveScopeDeclaration,
            &temporal_annotation,
        )
        .ok_or("Failed to add @Temporal annotation to field")?;
    }
    
    // Add @TimeZoneStorage annotation if needed
    if let Some(timezone_annotation) = generate_timezone_storage_annotation(field_config) {
        // Get updated field node position after previous annotations
        let updated_class_node = get_class_node(ts_file)?;
        let updated_field_node = field_declaration_service::find_field_declaration_node_by_name(
            ts_file,
            &field_config.field_name,
            updated_class_node,
        )
        .ok_or("Failed to find field for timezone annotation")?;
        
        annotation_service::add_annotation(
            ts_file,
            updated_field_node.start_byte(),
            &AnnotationInsertionPosition::AboveScopeDeclaration,
            &timezone_annotation,
        )
        .ok_or("Failed to add @TimeZoneStorage annotation to field")?;
    }
    
    // Add @Lob annotation if needed
    if let Some(lob_annotation) = generate_lob_annotation(field_config) {
        // Get updated field node position after previous annotations
        let updated_class_node = get_class_node(ts_file)?;
        let updated_field_node = field_declaration_service::find_field_declaration_node_by_name(
            ts_file,
            &field_config.field_name,
            updated_class_node,
        )
        .ok_or("Failed to find field for LOB annotation")?;
        
        annotation_service::add_annotation(
            ts_file,
            updated_field_node.start_byte(),
            &AnnotationInsertionPosition::AboveScopeDeclaration,
            &lob_annotation,
        )
        .ok_or("Failed to add @Lob annotation to field")?;
    }
    
    Ok(())
}

fn save_ts_file(ts_file: &mut TSFile, file_path: &Path) -> Result<(), String> {
    ts_file
        .save_as(file_path)
        .map_err(|_| "Failed to save file".to_string())
}

fn build_file_response(ts_file: &TSFile, entity_file_path: &Path) -> Result<FileResponse, String> {
    let file_type_str = ts_file
        .get_file_name_without_ext()
        .ok_or("Failed to get file type string")?;
    let file_path = entity_file_path.to_string_lossy().to_string();
    // Extract package name from file (this is a simplified approach)
    let file_package_name = "unknown".to_string(); // TODO: Extract actual package name from file
    
    Ok(FileResponse {
        file_type: file_type_str,
        file_path,
        file_package_name,
    })
}

pub fn run(
    _cwd: &Path,
    entity_file_path: &Path,
    field_config: BasicFieldConfig,
) -> Result<FileResponse, String> {
    // Step 1: Load the entity file as a TSFile
    let mut ts_file = load_ts_file(entity_file_path)?;
    
    // Step 2: Add required imports
    add_jpa_column_import(&mut ts_file)?;
    
    // Add temporal imports if temporal type is specified
    if field_config.field_temporal.is_some() {
        add_temporal_imports(&mut ts_file)?;
    }
    
    // Add timezone storage imports if timezone storage is specified
    if field_config.field_timezone_storage.is_some() {
        add_timezone_storage_imports(&mut ts_file)?;
    }
    
    // Add custom type import if package name is specified
    if let Some(package_name) = &field_config.field_type_package_name {
        add_custom_type_import(&mut ts_file, package_name, &field_config.field_type)?;
    }
    
    // Add LOB import if large object is specified
    if field_config.field_large_object == Some(true) {
        add_lob_import(&mut ts_file)?;
    }
    
    // Step 3: Get class node after imports (byte positions may have shifted)
    let updated_class_node = get_class_node(&ts_file)?;
    let class_byte_position = updated_class_node.start_byte();
    
    // Step 4: Add the field with all necessary annotations
    add_field_with_annotations(&mut ts_file, class_byte_position, &field_config)?;
    
    // Step 6: Save the updated TSFile to disk
    save_ts_file(&mut ts_file, entity_file_path)?;
    
    // Step 7: Build and return the file response
    build_file_response(&ts_file, entity_file_path)
}
