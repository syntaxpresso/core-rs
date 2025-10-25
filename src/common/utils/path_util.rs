use std::path::Path;

use walkdir::WalkDir;

use crate::common::ts_file::TSFile;

pub fn parse_all_files(cwd: &Path) -> Vec<TSFile> {
    let extension = "java";
    let mut files = Vec::new();
    for entry in WalkDir::new(cwd).into_iter().flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension()
            && ext.to_string_lossy().eq_ignore_ascii_case(extension)
            && let Ok(ts_file) = TSFile::from_file(path)
        {
            files.push(ts_file);
        }
    }
    files
}
