#[derive(Debug, Clone, PartialEq)]
pub enum EntitySide {
  /// Owning side of the relationship - contains foreign key, defines relationship
  Owning,
  /// Inverse side of the relationship - referenced by owning side, uses mappedBy
  Inverse,
}
