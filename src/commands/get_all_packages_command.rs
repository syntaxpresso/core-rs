use std::path::Path;

use crate::{
  commands::services::get_all_packages_service::run,
  common::types::java_source_directory_type::JavaSourceDirectoryType,
  responses::{get_packages_response::GetPackagesResponse, response::Response},
};

pub fn execute(
  cwd: &Path,
  source_directory: &JavaSourceDirectoryType,
) -> Response<GetPackagesResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("get-all-packages");
  match run(cwd, source_directory) {
    Ok(packages) => {
      let packages_count = packages.len();
      let root_package_name = packages
        .iter()
        .min_by_key(|package| package.package_name.len())
        .map(|r| r.package_name.clone());
      let response = GetPackagesResponse { packages, packages_count, root_package_name };
      Response::success(cmd_name, cwd_string, response)
    }
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
