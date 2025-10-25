use std::path::PathBuf;

use crate::{
    commands::services::get_all_files_service::run,
    responses::{get_all_files_response::GetAllFilesCommandResponse, response::Response},
};

pub fn execute(cwd: PathBuf) -> Response<GetAllFilesCommandResponse> {
    let cwd_string = cwd.display().to_string();
    match run(&cwd) {
        Ok(files) => {
            let files_count = files.len();
            let response = GetAllFilesCommandResponse { files, files_count };
            Response::success("get-all-files".to_string(), cwd_string, response)
        }
        Err(error_msg) => Response::error("get-all-files".to_string(), cwd_string, error_msg),
    }
}
