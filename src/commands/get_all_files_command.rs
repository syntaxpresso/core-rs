use std::path::PathBuf;

use crate::{
    common::utils::path_util::find_all_files_by_extension,
    responses::get_all_files_response::GetAllFilesCommandResponse,
};

pub fn execute(cwd: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let ext = "java";
    let files = find_all_files_by_extension(&cwd, ext);
    let files_size = files.len();
    let response = GetAllFilesCommandResponse {
        command: "get-all-files".to_string(),
        cwd: cwd.display().to_string(),
        files,
        files_count: files_size,
    };
    let json = serde_json::to_string_pretty(&response)?;
    Ok(json)
}
