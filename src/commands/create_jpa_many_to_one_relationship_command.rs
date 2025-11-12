use std::path::Path;

use crate::{
  commands::{
    services::create_jpa_many_to_one_relationship_service,
    validators::directory_validator::validate_file_path_within_base,
  },
  common::types::many_to_one_field_config::ManyToOneFieldConfig,
  responses::{get_files_response::GetFilesResponse, response::Response},
};

pub fn execute(
  cwd: &Path,
  owning_side_entity_file_b64_src: &str,
  owning_side_entity_file_path: &Path,
  owning_side_field_name: String,
  inverse_side_field_name: String,
  config: ManyToOneFieldConfig,
) -> Response<GetFilesResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("create-jpa-many-to-one-relationship");
  // Security validation: ensure owning side entity file path is within the cwd
  let file_path_str = owning_side_entity_file_path.display().to_string();
  if let Err(error_msg) = validate_file_path_within_base(&file_path_str, cwd) {
    return Response::error(
      cmd_name,
      cwd_string,
      format!("Owning side entity file path security validation failed: {}", error_msg),
    );
  }

  match create_jpa_many_to_one_relationship_service::run(
    cwd,
    owning_side_entity_file_b64_src,
    owning_side_entity_file_path,
    &owning_side_field_name,
    &inverse_side_field_name,
    &config,
  ) {
    Ok(files) => {
      let files_count = files.len();
      let response = GetFilesResponse { files, files_count };
      Response::success(cmd_name, cwd_string, response)
    }
    Err(e) => Response::error(cmd_name, cwd_string, e.to_string()),
  }
}
