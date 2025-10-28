use crate::common::types::basic_field_config::BasicFieldConfig;
use crate::responses::file_response::FileResponse;
use std::path::Path;

pub fn run(
    cwd: &Path,
    entity_file_path: &Path,
    field_config: BasicFieldConfig,
) -> Result<FileResponse, String> {
    // TODO: Implement JPA entity basic field creation logic
    Err("Not implemented yet".to_string())
}
