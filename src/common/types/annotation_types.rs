#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationInsertionPosition {
    BeforeFirstAnnotation,
    AboveScopeDeclaration,
}
