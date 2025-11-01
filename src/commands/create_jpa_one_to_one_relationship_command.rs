use std::path::Path;

use crate::{
  commands::services::create_jpa_one_to_one_relationship_service,
  common::types::one_to_one_field_config::OneToOneFieldConfig,
  responses::{get_files_response::GetFilesResponse, response::Response},
};

pub fn execute(
  cwd: &Path,
  owning_side_entity_file_path: &Path,
  owning_side_field_name: String,
  inverse_side_field_name: String,
  config: OneToOneFieldConfig,
) -> Response<GetFilesResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("create-jpa-one-to-one-relationship");
  match create_jpa_one_to_one_relationship_service::run(
    cwd,
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
