#![allow(dead_code)]

use crate::common::types::{
    java_id_generation::JavaIdGeneration,
    java_id_generation_type::JavaIdGenerationType,
};

#[derive(Debug, Clone)]
pub struct IdFieldConfig {
    pub field_name: String,
    pub field_type: String,
    pub field_type_package_name: Option<String>,
    pub field_id_generation: JavaIdGeneration,
    pub field_id_generation_type: Option<JavaIdGenerationType>,
    pub field_generator_name: Option<String>,
    pub field_sequence_name: Option<String>,
    pub field_initial_value: Option<i64>,
    pub field_allocation_size: Option<i64>,
    pub field_nullable: bool,
    pub field_mutable: bool,
}
