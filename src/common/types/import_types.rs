#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ImportInsertionPoint {
  pub position: ImportInsertionPosition,
  pub insert_byte: usize,
  pub break_line_before: bool,
  pub break_line_after: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportInsertionPosition {
  BeforeFirstImport,
  AfterLastImport,
  AfterPackageDeclaration,
}
