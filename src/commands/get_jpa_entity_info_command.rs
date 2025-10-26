use base64::prelude::*;
use std::fs;
use std::path::Path;

use crate::{
    commands::services::get_jpa_entity_info_service,
    responses::{get_jpa_entity_info_response::GetJpaEntityInfoResponse, response::Response},
};

pub fn execute(entity_file_path: &Path) -> Response<GetJpaEntityInfoResponse> {
    let cwd_string = entity_file_path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| String::from("."));
    let cmd_name = String::from("get-jpa-entity-info");
    // Read the entity file and encode it as base64
    let entity_content = match fs::read_to_string(entity_file_path) {
        Ok(content) => content,
        Err(e) => {
            return Response::error(
                cmd_name,
                cwd_string,
                format!("Failed to read entity file: {}", e),
            );
        }
    };
    let b64_entity_content = BASE64_STANDARD.encode(entity_content.as_bytes());
    let file_path_str = entity_file_path.to_str().unwrap_or("");
    match get_jpa_entity_info_service::run(&b64_entity_content, file_path_str) {
        Ok(response) => Response::success(cmd_name, cwd_string, response),
        Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
    }
}

