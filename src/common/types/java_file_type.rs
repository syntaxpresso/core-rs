#![allow(dead_code)]

use crate::common::utils::case_util;
use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JavaFileType {
    #[value(name = "class")]
    Class,

    #[value(name = "interface")]
    Interface,

    #[value(name = "enum")]
    Enum,

    #[value(name = "record")]
    Record,

    #[value(name = "annotation")]
    Annotation,
}

impl JavaFileType {
    /// Generate source content with package name and class name
    ///
    /// # Arguments
    /// * `package_name` - The Java package name (e.g., "com.example.entity")
    /// * `type_name` - The class/interface/enum name
    ///
    /// # Returns
    /// Generated Java source code as a String
    pub fn get_source_content(&self, package_name: &str, type_name: &str) -> String {
        // Ensure type name is in PascalCase for Java conventions
        let normalized_type_name = case_util::to_pascal_case(type_name);
        match self {
            JavaFileType::Class => {
                format!("package {};\n\npublic class {} {{}}", package_name, normalized_type_name)
            }
            JavaFileType::Interface => {
                format!(
                    "package {};\n\npublic interface {} {{}}",
                    package_name, normalized_type_name
                )
            }
            JavaFileType::Enum => {
                format!("package {};\n\npublic enum {} {{}}", package_name, normalized_type_name)
            }
            JavaFileType::Record => {
                format!(
                    "package {};\n\npublic record {}() {{}}",
                    package_name, normalized_type_name
                )
            }
            JavaFileType::Annotation => {
                format!(
                    "package {};\n\npublic @interface {} {{}}",
                    package_name, normalized_type_name
                )
            }
        }
    }

    /// Generate source content with additional parameters (for compatibility with Java version)
    ///
    /// # Arguments
    /// * `package_name` - The Java package name
    /// * `type_name` - The class/interface/enum name
    ///
    /// # Returns
    /// Generated Java source code as a String
    pub fn get_source_content_with_types(&self, package_name: &str, type_name: &str) -> String {
        // For now, delegate to the simpler version
        // This can be extended later if specific templates need the additional parameters
        self.get_source_content(package_name, type_name)
    }
}
