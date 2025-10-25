use std::path::{Path, PathBuf};

use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JavaSourceDirectoryType {
    #[value(name = "main")]
    Main,
    #[value(name = "test")]
    Test,
    #[value(name = "all")]
    All,
}

impl JavaSourceDirectoryType {
    /// Get the directory path for this source type
    pub fn get_directory_path(&self) -> &'static str {
        match self {
            JavaSourceDirectoryType::Main => "src/main/java",
            JavaSourceDirectoryType::Test => "src/test/java",
            JavaSourceDirectoryType::All => "src",
        }
    }

    /// Get the Maven/Gradle standard directory structure path
    pub fn get_full_path(&self, base_path: &Path, package_name: &str) -> PathBuf {
        let package_path = package_name.replace('.', "/");
        match self {
            JavaSourceDirectoryType::Main => base_path.join("src/main/java").join(&package_path),
            JavaSourceDirectoryType::Test => base_path.join("src/test/java").join(&package_path),
            JavaSourceDirectoryType::All => base_path.join("src").join(&package_path),
        }
    }
}
