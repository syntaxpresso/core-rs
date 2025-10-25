use std::path::PathBuf;

use crate::{
    commands::services::get_all_files_service::run,
    responses::{get_all_files_response::GetAllFilesCommandResponse, response::Response},
};

pub fn execute(cwd: PathBuf) -> Response<GetAllFilesCommandResponse> {
    let cwd_string = cwd.display().to_string();
    match std::panic::catch_unwind(|| {
        let files = run(&cwd);
        let files_size = files.len();
        GetAllFilesCommandResponse {
            files,
            files_count: files_size,
        }
    }) {
        Ok(response) => {
            Response::success("get-all-files".to_string(), cwd_string.clone(), response)
        }
        Err(_) => Response::error(
            "get-all-files".to_string(),
            cwd_string,
            "Failed to execute get-all-files command".to_string(),
        ),
    }
}
