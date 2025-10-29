#![allow(dead_code)]

use crate::common::types::{
    java_field_temporal::JavaFieldTemporal, java_field_time_zone_storage::JavaFieldTimeZoneStorage,
};

#[derive(Debug, Clone)]
pub struct BasicFieldConfig {
    pub field_name: String,
    pub field_type: String,
    pub field_type_package_name: Option<String>,
    pub field_length: Option<u16>,
    pub field_precision: Option<u16>,
    pub field_scale: Option<u16>,
    pub field_temporal: Option<JavaFieldTemporal>,
    pub field_timezone_storage: Option<JavaFieldTimeZoneStorage>,
    pub field_unique: bool,
    pub field_nullable: bool,
    pub field_large_object: bool,
}
