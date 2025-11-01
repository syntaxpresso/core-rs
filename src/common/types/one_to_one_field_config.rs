use crate::common::types::{
  cascade_type::CascadeType, mapping_type::MappingType, other_type::OtherType,
};

#[derive(Debug, Clone)]
pub struct OneToOneFieldConfig {
  pub inverse_field_type: String,
  pub mapping_type: Option<MappingType>,
  pub owning_side_cascades: Vec<CascadeType>,
  pub inverse_side_cascades: Vec<CascadeType>,
  pub owning_side_other: Vec<OtherType>,
  pub inverse_side_other: Vec<OtherType>,
}
