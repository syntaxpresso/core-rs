// use std::path::PathBuf;
//
// use crate::{
//     commands::services::get_all_files_service::run,
//     responses::{file_response::FileResponse, response::Response},
// };
//
// pub fn execute(cwd: PathBuf) -> Response<FileResponse> {
//     let cwd_string = cwd.display().to_string();
//     let cmd_name = String::from("create-java-file");
//     match run(&cwd) {
//         Ok(response) => Response::success(cmd_name, cwd_string, response),
//         Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
//     }
// }
