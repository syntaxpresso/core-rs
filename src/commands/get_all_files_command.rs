use std::path::Path;

use crate::{
  commands::services::get_all_files_service::run,
  responses::{get_all_files_response::GetAllFilesCommandResponse, response::Response},
};

pub fn execute(cwd: &Path) -> Response<GetAllFilesCommandResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("get-all-files");
  match run(cwd) {
    Ok(files) => {
      let files_count = files.len();
      let response = GetAllFilesCommandResponse { files, files_count };
      Response::success(cmd_name, cwd_string, response)
    }
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
