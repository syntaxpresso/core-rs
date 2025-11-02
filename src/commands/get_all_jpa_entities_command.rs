use std::path::Path;

use crate::{
  commands::services::get_all_jpa_entities_service::run,
  responses::{get_files_response::GetFilesResponse, response::Response},
};

pub fn execute(cwd: &Path) -> Response<GetFilesResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("get-all-jpa-entities");
  match run(cwd) {
    Ok(files) => {
      let files_count = files.len();
      let response = GetFilesResponse { files, files_count };
      Response::success(cmd_name, cwd_string, response)
    }
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
