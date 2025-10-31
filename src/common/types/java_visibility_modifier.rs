#![allow(dead_code)]

/// Represents Java visibility modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaVisibilityModifier {
  /// Public visibility - accessible from anywhere
  Public,
  /// Private visibility - accessible only within the same class
  Private,
  /// Protected visibility - accessible within package and subclasses
  Protected,
  /// Package-private visibility - no explicit modifier keyword
  PackagePrivate,
}

impl JavaVisibilityModifier {
  /// Gets the Java keyword for this visibility modifier.
  /// Returns empty string for package-private.
  pub fn keyword(&self) -> &'static str {
    match self {
      JavaVisibilityModifier::Public => "public",
      JavaVisibilityModifier::Private => "private",
      JavaVisibilityModifier::Protected => "protected",
      JavaVisibilityModifier::PackagePrivate => "",
    }
  }

  /// Checks if this modifier has an explicit keyword.
  pub fn has_keyword(&self) -> bool {
    !self.keyword().is_empty()
  }
}

impl std::fmt::Display for JavaVisibilityModifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self.keyword() {
      "" => "package-private",
      kw => kw,
    };
    write!(f, "{}", s)
  }
}
