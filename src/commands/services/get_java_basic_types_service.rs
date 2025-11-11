use crate::{
  common::types::java_basic_types::JavaBasicType,
  responses::basic_java_type_response::JavaBasicTypeResponse,
};

pub fn run(basic_type_kind: &JavaBasicType) -> Result<Vec<JavaBasicTypeResponse>, String> {
  let types = match basic_type_kind {
    JavaBasicType::AllTypes => basic_type_kind.get_all_types(),
    JavaBasicType::IdTypes => basic_type_kind.get_id_types(),
    JavaBasicType::TypesWithLength => basic_type_kind.get_types_with_length(),
    JavaBasicType::TypesWithTimeZoneStorage => basic_type_kind.get_types_with_time_zone_storage(),
    JavaBasicType::TypesWithTemporal => basic_type_kind.get_types_with_temporal(),
    JavaBasicType::TypesWithExtraOther => basic_type_kind.get_types_with_extra_other(),
    JavaBasicType::TypesWithPrecisionAndScale => basic_type_kind.get_types_with_precision_scale(),
  };
  Ok(types)
}
