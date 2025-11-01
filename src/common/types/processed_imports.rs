/// Common import processing result for JPA relationship services
/// 
/// This struct is used by both OneToOne and ManyToOne relationship services
/// to hold the processed import information that will be added to entity files.
/// 
/// The imports are stored as tuples of (package_name, class_name) to be used
/// with the import_declaration_service.
#[derive(Debug, Clone)]
pub struct ProcessedImports {
  /// Entity class import (package, class_name) - e.g., ("com.example.entity", "User")
  pub entity_class_import: Option<(String, String)>,
  
  /// JPA and utility imports (package, class_name) - e.g., ("jakarta.persistence", "OneToOne")
  pub jpa_imports: Vec<(String, String)>,
}

impl ProcessedImports {
  /// Create a new empty ProcessedImports
  pub fn new() -> Self {
    Self {
      entity_class_import: None,
      jpa_imports: Vec::new(),
    }
  }
  
  /// Add an entity class import
  pub fn set_entity_import(&mut self, package: String, class_name: String) {
    self.entity_class_import = Some((package, class_name));
  }
  
  /// Add a JPA import
  pub fn add_jpa_import(&mut self, package: String, class_name: String) {
    self.jpa_imports.push((package, class_name));
  }
  
  /// Get all imports as a single iterator
  pub fn all_imports(&self) -> impl Iterator<Item = &(String, String)> {
    self.entity_class_import.iter().chain(self.jpa_imports.iter())
  }
}

impl Default for ProcessedImports {
  fn default() -> Self {
    Self::new()
  }
}