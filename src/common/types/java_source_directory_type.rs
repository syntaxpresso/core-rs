#[derive(Debug, Clone, PartialEq)]
pub enum JavaSourceDirectoryType {
    Main,
    Test,
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
    pub fn get_full_path(&self, base_path: &str, package_name: &str) -> String {
        let package_path = package_name.replace('.', "/");
        match self {
            JavaSourceDirectoryType::Main => {
                format!("{}/src/main/java/{}", base_path, package_path)
            }
            JavaSourceDirectoryType::Test => {
                format!("{}/src/test/java/{}", base_path, package_path)
            }
            JavaSourceDirectoryType::All => {
                format!("{}/src/{}", base_path, package_path)
            }
        }
    }
}