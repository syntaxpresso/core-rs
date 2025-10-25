use std::path::PathBuf;

pub fn validate_directory(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() && path.is_dir() {
        Ok(path)
    } else {
        Err(format!("Directory does not exist: {}", s))
    }
}
