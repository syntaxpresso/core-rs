use crate::common::types::cascade_type::CascadeType;
use crate::common::types::collection_type::CollectionType;
use crate::common::types::fetch_type::FetchType;
use crate::common::types::other_type::OtherType;

/// Common annotation configuration for JPA relationship services
/// 
/// This struct is used by both OneToOne and ManyToOne relationship services
/// to configure the annotations that will be added to entity fields.
/// 
/// For OneToOne relationships:
/// - `fetch_type` and `collection_type` should be `None`
/// 
/// For ManyToOne relationships:
/// - `fetch_type` and `collection_type` are used for the owning and inverse sides respectively
#[derive(Debug, Clone)]
pub struct AnnotationConfig {
  #[allow(dead_code)]
  pub is_owning_side: bool,
  pub cascades: Vec<CascadeType>,
  pub other_options: Vec<OtherType>,
  pub mapped_by_field: Option<String>,
  pub needs_join_column: bool,
  
  // Optional fields for ManyToOne relationships
  pub fetch_type: Option<FetchType>,
  pub collection_type: Option<CollectionType>,
}

impl AnnotationConfig {
  /// Create a new AnnotationConfig for OneToOne relationships
  pub fn new_one_to_one(
    is_owning_side: bool,
    cascades: Vec<CascadeType>,
    other_options: Vec<OtherType>,
    mapped_by_field: Option<String>,
    needs_join_column: bool,
  ) -> Self {
    Self {
      is_owning_side,
      cascades,
      other_options,
      mapped_by_field,
      needs_join_column,
      fetch_type: None,
      collection_type: None,
    }
  }

  /// Create a new AnnotationConfig for ManyToOne relationships
  pub fn new_many_to_one(
    is_owning_side: bool,
    cascades: Vec<CascadeType>,
    other_options: Vec<OtherType>,
    mapped_by_field: Option<String>,
    needs_join_column: bool,
    fetch_type: FetchType,
    collection_type: CollectionType,
  ) -> Self {
    Self {
      is_owning_side,
      cascades,
      other_options,
      mapped_by_field,
      needs_join_column,
      fetch_type: Some(fetch_type),
      collection_type: Some(collection_type),
    }
  }
  
  /// Get the fetch type, or None if not applicable
  pub fn get_fetch_type(&self) -> Option<&FetchType> {
    self.fetch_type.as_ref()
  }
  
  /// Get the collection type, or None if not applicable
  pub fn get_collection_type(&self) -> Option<&CollectionType> {
    self.collection_type.as_ref()
  }
}