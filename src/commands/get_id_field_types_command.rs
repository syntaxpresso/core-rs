use std::path::Path;

use crate::{
  commands::services::get_id_field_types_service::run,
  responses::{basic_java_type::BasicJavaType, response::Response},
};

pub fn execute(cwd: &Path) -> Response<Vec<BasicJavaType>> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("get-id-field-types");
  match run() {
    Ok(types) => Response::success(cmd_name, cwd_string, types),
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
