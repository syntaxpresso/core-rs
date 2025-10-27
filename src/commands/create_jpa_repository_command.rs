use std::path::Path;

use crate::{
    commands::services::create_jpa_repository_service::run,
    responses::{create_jpa_repository_response::CreateJPARepositoryResponse, response::Response},
};

pub fn execute(
    cwd: &Path,
    file_path: &Path,
    b64_superclass_source: Option<&str>,
) -> Response<CreateJPARepositoryResponse> {
    let cwd_string = cwd.display().to_string();
    let cmd_name = String::from("create-jpa-repository");
    match run(cwd, file_path, b64_superclass_source) {
        Ok(response) => Response::success(cmd_name, cwd_string, response),
        Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
    }
}
