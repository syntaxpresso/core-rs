use std::path::Path;

use crate::{
  common::{
    ts_file::TSFile,
    types::{java_file_type::JavaFileType, java_source_directory_type::JavaSourceDirectoryType},
    utils::{case_util, path_security_util::PathSecurityValidator},
  },
  responses::file_response::FileResponse,
};

pub fn generate_file_template(
  file_type: &JavaFileType,
  package_name: &str,
  file_name: &str,
) -> String {
  file_type.get_source_content(package_name, file_name)
}

pub fn create_ts_file(file_template: &str) -> TSFile {
  TSFile::from_source_code(file_template)
}

pub fn correct_java_file_name(file_name: &str) -> String {
  match std::path::Path::new(file_name).extension() {
    Some(ext) if ext == "java" => file_name.to_string(),
    _ => match std::path::Path::new(file_name).file_stem() {
      Some(stem) => {
        format!(
          "{}.java",
          case_util::auto_convert_case(&stem.to_string_lossy(), case_util::CaseType::Pascal)
        )
      }
      None => "Unknown.java".to_string(),
    },
  }
}

pub fn build_save_path(
  source_directory: &JavaSourceDirectoryType,
  cwd: &Path,
  package_name: &str,
  corrected_file_name: &str,
) -> Result<std::path::PathBuf, String> {
  // Create path security validator for the current working directory
  let validator = PathSecurityValidator::new(cwd)?;
  // Build the intended path using the standard Java directory structure
  let intended_path = source_directory.get_full_path(cwd, package_name).join(corrected_file_name);
  // Validate that the path is contained within the working directory
  validator
    .validate_path_containment(&intended_path)
    .map_err(|e| format!("Path security validation failed: {}", e))
}

fn save_ts_file(
  ts_file: &mut TSFile,
  save_path: &std::path::Path,
  base_path: &Path,
) -> Result<(), String> {
  ts_file.save_as(save_path, base_path).map_err(|e| format!("Failed to save file: {}", e))
}

fn build_file_response(ts_file: &TSFile, package_name: &str) -> Result<FileResponse, String> {
  let file_type_str =
    ts_file.get_file_name_without_ext().ok_or("Failed to get file type string")?;
  let file_path = ts_file
    .file_path()
    .map(|p| p.to_string_lossy().to_string())
    .ok_or("Failed to get file path")?;
  let file_package_name = package_name.to_string();
  Ok(FileResponse { file_type: file_type_str, file_path, file_package_name })
}

pub fn run(
  cwd: &Path,
  package_name: &str,
  file_name: &str,
  file_type: &JavaFileType,
  source_directory: &JavaSourceDirectoryType,
) -> Result<FileResponse, String> {
  // Step 1: Generate file template
  let file_template = generate_file_template(file_type, package_name, file_name);
  // Step 2: Create TSFile
  let mut ts_file = create_ts_file(&file_template);
  // Step 3: Correct file name
  let corrected_file_name = correct_java_file_name(file_name);
  // Step 4: Build save path with security validation
  let save_path = build_save_path(source_directory, cwd, package_name, &corrected_file_name)?;
  // Step 5: Check if file exists before saving
  if save_path.exists() {
    return Err(format!("File already exists: {}", save_path.display()));
  }
  save_ts_file(&mut ts_file, &save_path, cwd)?;
  // Step 6: Build response
  build_file_response(&ts_file, package_name)
}
