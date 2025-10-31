/// Represents Java field modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaFieldModifier {
  /// Static modifier - field belongs to the class
  Static,
  /// Final modifier - field cannot be reassigned
  Final,
  /// Transient modifier - field is not serialized
  Transient,
  /// Volatile modifier - field access is thread-safe
  Volatile,
}

impl JavaFieldModifier {
  /// Gets the Java keyword for this field modifier.
  pub fn keyword(&self) -> &'static str {
    match self {
      JavaFieldModifier::Static => "static",
      JavaFieldModifier::Final => "final",
      JavaFieldModifier::Transient => "transient",
      JavaFieldModifier::Volatile => "volatile",
    }
  }
}

impl std::fmt::Display for JavaFieldModifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.keyword())
  }
}
