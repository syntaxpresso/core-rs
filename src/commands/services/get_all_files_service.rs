use std::path::PathBuf;

use walkdir::WalkDir;

use crate::responses::get_all_files_response::FileResponse;

pub fn run(cwd: &PathBuf, extension: &str) -> Vec<FileResponse> {
    let mut files = Vec::new();
    for entry in WalkDir::new(cwd).into_iter().flatten() {
        let path = entry.path();
        if let (Some(ext), Some(stem)) = (path.extension(), path.file_stem())
            && ext.to_string_lossy().eq_ignore_ascii_case(extension)
        {
            let found_file = FileResponse {
                file_type: stem.to_string_lossy().to_string(),
                file_path: path.display().to_string(),
            };
            files.push(found_file);
        }
    }
    files
}
