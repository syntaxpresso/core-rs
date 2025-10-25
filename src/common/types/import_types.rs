#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum ImportInsertionPosition {
    BeforeFirstImport,
    AfterLastImport,
    AfterPackageDeclaration,
}