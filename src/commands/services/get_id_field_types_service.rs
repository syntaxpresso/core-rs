use crate::responses::basic_java_type::BasicJavaType;

pub fn run() -> Result<Vec<BasicJavaType>, String> {
  let types = vec![
    BasicJavaType {
      id: "java.lang.Long".into(),
      name: "Long".into(),
      package_path: "java.lang".into(),
      type_: "Long".into(),
    },
    BasicJavaType {
      id: "java.lang.Integer".into(),
      name: "Integer".into(),
      package_path: "java.lang".into(),
      type_: "Integer".into(),
    },
    BasicJavaType {
      id: "java.lang.String".into(),
      name: "String".into(),
      package_path: "java.lang".into(),
      type_: "String".into(),
    },
    BasicJavaType {
      id: "java.util.UUID".into(),
      name: "UUID".into(),
      package_path: "java.util".into(),
      type_: "UUID".into(),
    },
  ];
  Ok(types)
}
