use clap::ValueEnum;

use crate::responses::basic_java_type_response::JavaBasicTypeResponse;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JavaBasicType {
  #[value(name = "all")]
  All,
  #[value(name = "id")]
  Id,
  #[value(name = "types-with-length")]
  TypesWithLength,
  #[value(name = "types-with-time-zone-storage")]
  TypesWithTimeZoneStorage,
  #[value(name = "types-with-temporal")]
  TypesWithTemporal,
  #[value(name = "types-with-extra-other")]
  TypesWithExtraOther,
  #[value(name = "types-with-precision-and-scale")]
  TypesWithPrecisionAndScale,
}

impl JavaBasicType {
  pub fn get_all(&self) -> Vec<JavaBasicTypeResponse> {
    vec![
      JavaBasicTypeResponse {
        id: "java.lang.String".into(),
        name: "String".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Long".into(),
        name: "Long".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Integer".into(),
        name: "Integer".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Boolean".into(),
        name: "Boolean".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Double".into(),
        name: "Double".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.math.BigDecimal".into(),
        name: "BigDecimal".into(),
        package_path: Some("java.math".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.Instant".into(),
        name: "Instant".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.LocalDateTime".into(),
        name: "LocalDateTime".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.LocalDate".into(),
        name: "LocalDate".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.LocalTime".into(),
        name: "LocalTime".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.OffsetDateTime".into(),
        name: "OffsetDateTime".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.OffsetTime".into(),
        name: "OffsetTime".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.Date".into(),
        name: "Date (util)".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.Date".into(),
        name: "Date (sql)".into(),
        package_path: Some("java.sql".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.Time".into(),
        name: "Time".into(),
        package_path: Some("java.sql".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.Timestamp".into(),
        name: "Timestamp".into(),
        package_path: Some("java.sql".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.TimeZone".into(),
        name: "TimeZone".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Byte[]".into(),
        name: "Byte[]".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.Blob".into(),
        name: "Blob".into(),
        package_path: Some("java.sql".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Byte".into(),
        name: "Byte".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Character".into(),
        name: "Character".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Short".into(),
        name: "Short".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Float".into(),
        name: "Float".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.math.BigInteger".into(),
        name: "BigInteger".into(),
        package_path: Some("java.math".into()),
      },
      JavaBasicTypeResponse {
        id: "java.net.URL".into(),
        name: "URL".into(),
        package_path: Some("java.net".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.Duration".into(),
        name: "Duration".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.ZonedDateTime".into(),
        name: "ZonedDateTime".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.Calendar".into(),
        name: "Calendar".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.Locale".into(),
        name: "Locale".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.Currency".into(),
        name: "Currency".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Class".into(),
        name: "Class".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.UUID".into(),
        name: "UUID".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Character[]".into(),
        name: "Character[]".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.Clob".into(),
        name: "Clob".into(),
        package_path: Some("java.sql".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.NClob".into(),
        name: "NClob".into(),
        package_path: Some("java.sql".into()),
      },
      // --- Primitives Start Here ---
      JavaBasicTypeResponse { id: "boolean".into(), name: "boolean".into(), package_path: None },
      JavaBasicTypeResponse { id: "byte".into(), name: "byte".into(), package_path: None },
      JavaBasicTypeResponse { id: "float".into(), name: "float".into(), package_path: None },
      JavaBasicTypeResponse { id: "char".into(), name: "char".into(), package_path: None },
      JavaBasicTypeResponse { id: "int".into(), name: "int".into(), package_path: None },
      JavaBasicTypeResponse { id: "double".into(), name: "double".into(), package_path: None },
      JavaBasicTypeResponse { id: "short".into(), name: "short".into(), package_path: None },
      JavaBasicTypeResponse { id: "long".into(), name: "long".into(), package_path: None },
      JavaBasicTypeResponse { id: "byte[]".into(), name: "byte[]".into(), package_path: None },
      JavaBasicTypeResponse { id: "char[]".into(), name: "char[]".into(), package_path: None },
      // --- Other Types ---
      JavaBasicTypeResponse {
        id: "org.geolatte.geom.Geometry".into(),
        name: "Geometry (geolatte)".into(),
        package_path: Some("org.geolatte.geom".into()),
      },
      JavaBasicTypeResponse {
        id: "com.vividsolutions.jts.geom.Geometry".into(),
        name: "Geometry (jts)".into(),
        package_path: Some("com.vividsolutions.jts.geom".into()),
      },
      JavaBasicTypeResponse {
        id: "java.net.InetAddress".into(),
        name: "InetAddress".into(),
        package_path: Some("java.net".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.ZoneOffset".into(),
        name: "ZoneOffset".into(),
        package_path: Some("java.time".into()),
      },
    ]
  }

  pub fn get_id_types(&self) -> Vec<JavaBasicTypeResponse> {
    vec![
      JavaBasicTypeResponse {
        id: "java.lang.Long".into(),
        name: "Long".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Integer".into(),
        name: "Integer".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.String".into(),
        name: "String".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.UUID".into(),
        name: "UUID".into(),
        package_path: Some("java.util".into()),
      },
    ]
  }

  pub fn get_types_with_length(&self) -> Vec<JavaBasicTypeResponse> {
    vec![
      JavaBasicTypeResponse {
        id: "java.lang.String".into(),
        name: "String".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.net.URL".into(),
        name: "URL".into(),
        package_path: Some("java.net".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.Locale".into(),
        name: "Locale".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.util.Currency".into(),
        name: "Currency".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Class".into(),
        name: "Class".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Character[]".into(),
        name: "Character[]".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse { id: "char[]".into(), name: "char[]".into(), package_path: None },
      JavaBasicTypeResponse {
        id: "java.util.TimeZone".into(),
        name: "TimeZone".into(),
        package_path: Some("java.util".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.ZoneOffset".into(),
        name: "ZoneOffset".into(),
        package_path: Some("java.time".into()),
      },
    ]
  }

  pub fn get_types_with_time_zone_storage(&self) -> Vec<JavaBasicTypeResponse> {
    vec![
      JavaBasicTypeResponse {
        id: "java.time.OffsetDateTime".into(),
        name: "OffsetDateTime".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.OffsetTime".into(),
        name: "OffsetTime".into(),
        package_path: Some("java.time".into()),
      },
      JavaBasicTypeResponse {
        id: "java.time.ZonedDateTime".into(),
        name: "ZonedDateTime".into(),
        package_path: Some("java.time".into()),
      },
    ]
  }

  pub fn get_types_with_temporal(&self) -> Vec<JavaBasicTypeResponse> {
    vec![JavaBasicTypeResponse {
      id: "java.util.Calendar".into(),
      name: "Calendar".into(),
      package_path: Some("java.util".into()),
    }]
  }

  pub fn get_types_with_extra_other(&self) -> Vec<JavaBasicTypeResponse> {
    vec![
      JavaBasicTypeResponse {
        id: "java.lang.String".into(),
        name: "String".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.lang.Byte[]".into(),
        name: "Byte[]".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse { id: "byte[]".into(), name: "byte[]".into(), package_path: None },
      JavaBasicTypeResponse { id: "char[]".into(), name: "char[]".into(), package_path: None },
      JavaBasicTypeResponse {
        id: "java.lang.Character[]".into(),
        name: "Character[]".into(),
        package_path: Some("java.lang".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.Blob".into(),
        name: "Blob".into(),
        package_path: Some("java.sql".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.Clob".into(),
        name: "Clob".into(),
        package_path: Some("java.sql".into()),
      },
      JavaBasicTypeResponse {
        id: "java.sql.NClob".into(),
        name: "NClob".into(),
        package_path: Some("java.sql".into()),
      },
    ]
  }

  pub fn get_types_with_precision_scale(&self) -> Vec<JavaBasicTypeResponse> {
    vec![JavaBasicTypeResponse {
      id: "java.math.BigDecimal".into(),
      name: "BigDecimal".into(),
      package_path: Some("java.math".into()),
    }]
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldInsertionPosition {
  AfterLastField,
  BeforeFirstMethod,
  EndOfClassBody,
}

#[derive(Debug, Clone)]
pub struct FieldInsertionPoint {
  pub position: FieldInsertionPosition,
  pub insert_byte: usize,
  pub break_line_before: bool,
  pub break_line_after: bool,
}

impl Default for FieldInsertionPoint {
  fn default() -> Self {
    Self::new()
  }
}

impl FieldInsertionPoint {
  pub fn new() -> Self {
    Self {
      position: FieldInsertionPosition::AfterLastField,
      insert_byte: 0,
      break_line_before: false,
      break_line_after: false,
    }
  }
}
