use std::path::Path;

use crate::{
  commands::services::get_java_files_service::run,
  common::types::java_file_type::JavaFileType,
  responses::{get_files_response::GetFilesResponse, response::Response},
};

pub fn execute(cwd: &Path, file_type: &JavaFileType) -> Response<GetFilesResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("get-java-files");
  match run(cwd, file_type) {
    Ok(files) => {
      let files_count = files.len();
      let response = GetFilesResponse { files, files_count };
      Response::success(cmd_name, cwd_string, response)
    }
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
