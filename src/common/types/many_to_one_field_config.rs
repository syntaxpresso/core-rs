use crate::common::types::{
  cascade_type::CascadeType, collection_type::CollectionType, fetch_type::FetchType,
  mapping_type::MappingType, other_type::OtherType,
};

#[derive(Debug, Clone)]
pub struct ManyToOneFieldConfig {
  pub inverse_field_type: String,
  pub fetch_type: FetchType,
  pub collection_type: CollectionType,
  pub mapping_type: Option<MappingType>,
  pub owning_side_cascades: Vec<CascadeType>,
  pub inverse_side_cascades: Vec<CascadeType>,
  pub owning_side_other: Vec<OtherType>,
  pub inverse_side_other: Vec<OtherType>,
}
