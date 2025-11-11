use std::path::Path;

use crate::{
  commands::services::get_java_basic_types_service::run,
  common::types::field_types::JavaBasicFieldTypeKind,
  responses::{basic_java_type::BasicJavaType, response::Response},
};

pub fn execute(
  cwd: &Path,
  field_type_kind: &JavaBasicFieldTypeKind,
) -> Response<Vec<BasicJavaType>> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("get-java-basic-types");
  match run(field_type_kind) {
    Ok(types) => Response::success(cmd_name, cwd_string, types),
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
