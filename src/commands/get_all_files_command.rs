use std::path::PathBuf;

use crate::{
    commands::services::get_all_files_service::run,
    responses::get_all_files_response::GetAllFilesCommandResponse,
};

pub fn execute(cwd: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let ext = "java";
    let files = run(&cwd, ext);
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
