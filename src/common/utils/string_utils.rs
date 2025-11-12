use base64::Engine;

pub fn decode_base64_or_invalid(base64_str: &str) -> String {
  match base64::engine::general_purpose::STANDARD.decode(base64_str) {
    Ok(bytes) => match String::from_utf8(bytes) {
      Ok(s) => s,
      Err(_) => "Invalid source code".to_string(),
    },
    Err(_) => "Invalid source code".to_string(),
  }
}
