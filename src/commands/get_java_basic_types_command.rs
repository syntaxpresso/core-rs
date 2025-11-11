use std::path::Path;

use crate::{
  commands::services::get_java_basic_types_service::run,
  common::types::java_basic_types::JavaBasicType,
  responses::{basic_java_type_response::JavaBasicTypeResponse, response::Response},
};

pub fn execute(
  cwd: &Path,
  basic_type_kind: &JavaBasicType,
) -> Response<Vec<JavaBasicTypeResponse>> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("get-java-basic-types");
  match run(basic_type_kind) {
    Ok(types) => Response::success(cmd_name, cwd_string, types),
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
