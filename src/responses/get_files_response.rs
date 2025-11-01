use serde::Serialize;

use crate::responses::file_response::FileResponse;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFilesResponse {
  pub files: Vec<FileResponse>,
  pub files_count: usize,
}
