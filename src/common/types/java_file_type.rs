#![allow(dead_code)]

use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JavaFileType {
    #[value(name = "class")]
    Class,

    #[value(name = "class")]
    Interface,

    #[value(name = "class")]
    Enum,

    #[value(name = "class")]
    Record,

    #[value(name = "class")]
    Annotation,
}

impl JavaFileType {
    /// Generate source content with package name and class name
    ///
    /// # Arguments
    /// * `package_name` - The Java package name (e.g., "com.example.entity")
    /// * `class_name` - The class/interface/enum name
    ///
    /// # Returns
    /// Generated Java source code as a String
    pub fn get_source_content(&self, package_name: &str, class_name: &str) -> String {
        match self {
            JavaFileType::Class => {
                format!(
                    "package {};\n\npublic class {} {{}}",
                    package_name, class_name
                )
            }
            JavaFileType::Interface => {
                format!(
                    "package {};\n\npublic interface {} {{}}",
                    package_name, class_name
                )
            }
            JavaFileType::Enum => {
                format!(
                    "package {};\n\npublic enum {} {{}}",
                    package_name, class_name
                )
            }
            JavaFileType::Record => {
                format!(
                    "package {};\n\npublic record {}() {{}}",
                    package_name, class_name
                )
            }
            JavaFileType::Annotation => {
                format!(
                    "package {};\n\npublic @interface {} {{}}",
                    package_name, class_name
                )
            }
        }
    }

    /// Generate source content with additional parameters (for compatibility with Java version)
    ///
    /// # Arguments
    /// * `package_name` - The Java package name
    /// * `class_name` - The class/interface/enum name
    ///
    /// # Returns
    /// Generated Java source code as a String
    pub fn get_source_content_with_types(&self, package_name: &str, class_name: &str) -> String {
        // For now, delegate to the simpler version
        // This can be extended later if specific templates need the additional parameters
        self.get_source_content(package_name, class_name)
    }
}
