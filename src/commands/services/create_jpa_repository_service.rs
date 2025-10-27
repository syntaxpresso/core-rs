// use base64::prelude::*;
// use std::fs;
// use std::path::Path;
//
// use crate::commands::services::get_jpa_entity_info_service;
// use crate::responses::file_response::FileResponse;
//
// fn prepare_repository_data(
//     entity_type: &str,
//     entity_package: &str,
//     id_field_type: &str,
//     id_field_package: &str,
// ) -> Result<RepositoryData, String> {
//     let repository_name = format!("{}Repository", entity_type);
//     let repository_package = entity_package.to_string();
//
//     Ok(RepositoryData {
//         repository_name,
//         repository_package,
//         entity_type: entity_type.to_string(),
//         entity_package: entity_package.to_string(),
//         id_field_type: id_field_type.to_string(),
//         id_field_package: id_field_package.to_string(),
//     })
// }
//
// fn create_repository_file(
//     cwd: &Path,
//     _entity_file_path: &Path,
//     repository_data: RepositoryData,
// ) -> Result<FileResponse, String> {
//     // Generate complete repository content
//     let repository_content = generate_repository_content(&repository_data);
//
//     // Build the file path manually
//     let package_path = repository_data.repository_package.replace('.', "/");
//     let src_main_java = cwd.join("src").join("main").join("java");
//     let package_dir = src_main_java.join(&package_path);
//     let file_path = package_dir.join(format!("{}.java", &repository_data.repository_name));
//
//     // Create directories if they don't exist
//     if let Some(parent) = file_path.parent() {
//         std::fs::create_dir_all(parent)
//             .map_err(|e| format!("Failed to create directories: {}", e))?;
//     }
//
//     // Check if file already exists
//     if file_path.exists() {
//         return Err(format!(
//             "Repository file already exists: {}",
//             file_path.display()
//         ));
//     }
//
//     // Write the repository file
//     std::fs::write(&file_path, repository_content)
//         .map_err(|e| format!("Failed to write repository file: {}", e))?;
//
//     // Return response
//     Ok(FileResponse {
//         file_type: repository_data.repository_name.clone(),
//         file_package_name: repository_data.repository_package.clone(),
//         file_path: file_path.to_string_lossy().to_string(),
//     })
// }
//
// fn generate_repository_content(data: &RepositoryData) -> String {
//     let mut imports = Vec::new();
//
//     // Add JpaRepository import
//     imports.push("import org.springframework.data.jpa.repository.JpaRepository;".to_string());
//
//     // Add entity import if different package
//     if data.entity_package != data.repository_package {
//         imports.push(format!(
//             "import {}.{};",
//             data.entity_package, data.entity_type
//         ));
//     }
//     // Add ID field type import if not a basic type
//     if !data.id_field_package.is_empty() && data.id_field_package != "java.lang" {
//         imports.push(format!(
//             "import {}.{};",
//             data.id_field_package, data.id_field_type
//         ));
//     }
//     let imports_section = if imports.is_empty() {
//         String::new()
//     } else {
//         format!("{}\n\n", imports.join("\n"))
//     };
//     format!(
//         "package {};\n\n{}public interface {} extends JpaRepository<{}, {}> {{\n}}\n",
//         data.repository_package,
//         imports_section,
//         data.repository_name,
//         data.entity_type,
//         data.id_field_type
//     )
// }
//
// #[derive(Debug)]
// struct RepositoryData {
//     repository_name: String,
//     repository_package: String,
//     entity_type: String,
//     entity_package: String,
//     id_field_type: String,
//     id_field_package: String,
// }
//
// pub fn run(
//     cwd: &Path,
//     entity_file_path: &Path,
//     b64_superclass_source: Option<&str>,
// ) -> Result<FileResponse, String> {
//     // Read the entity file and encode it as base64
//     let entity_content = fs::read_to_string(entity_file_path)
//         .map_err(|e| format!("Failed to read entity file: {}", e))?;
//
//     let b64_entity_content = base64::prelude::BASE64_STANDARD.encode(entity_content.as_bytes());
//
//     // Get entity information using our GetJpaEntityInfo service
//     let entity_info = get_jpa_entity_info_service::run(
//         &b64_entity_content,
//         entity_file_path.to_str().unwrap_or(""),
//     )?;
//
//     if !entity_info.is_jpa_entity {
//         return Err("The provided file is not a JPA entity".to_string());
//     }
//
//     // Handle missing ID field - check superclass if provided
//     let (id_field_type, id_field_package) = match (
//         &entity_info.id_field_type,
//         &entity_info.id_field_package_name,
//     ) {
//         (Some(field_type), Some(field_package)) => (field_type.clone(), field_package.clone()),
//         (None, None) => {
//             // If no ID field found and we have superclass source, check it
//             if let Some(b64_superclass) = b64_superclass_source {
//                 let superclass_info = get_jpa_entity_info_service::run(b64_superclass, "")?;
//                 match (
//                     &superclass_info.id_field_type,
//                     &superclass_info.id_field_package_name,
//                 ) {
//                     (Some(field_type), Some(field_package)) => {
//                         (field_type.clone(), field_package.clone())
//                     }
//                     _ => return Err("No @Id field found in entity or its superclass".to_string()),
//                 }
//             } else {
//                 return Err("No @Id field found in entity and no superclass provided".to_string());
//             }
//         }
//         _ => return Err("Incomplete ID field information".to_string()),
//     };
//
//     // Prepare repository data
//     let repository_data = prepare_repository_data(
//         &entity_info.entity_type,
//         &entity_info.entity_package_name,
//         &id_field_type,
//         &id_field_package,
//     )?;
//
//     // Create repository file
//     let repository_file_response = create_repository_file(cwd, entity_file_path, repository_data)?;
//
//     Ok(repository_file_response)
// }
