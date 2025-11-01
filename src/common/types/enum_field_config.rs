#![allow(dead_code)]

use crate::common::types::java_enum_type::JavaEnumType;

#[derive(Debug, Clone)]
pub struct EnumFieldConfig {
  pub field_name: String,
  pub enum_type: String,
  pub enum_package_name: String,
  pub enum_type_storage: JavaEnumType,
  pub field_length: Option<u16>,
  pub field_nullable: bool,
  pub field_unique: bool,
}