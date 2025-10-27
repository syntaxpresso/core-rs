#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationInsertionPosition {
    BeforeFirstAnnotation,
    AboveScopeDeclaration,
}

#[derive(Debug, Clone)]
pub struct AnnotationInsertionPoint {
    pub position: AnnotationInsertionPosition,
    pub insert_byte: usize,
    pub break_line_before: bool,
    pub break_line_after: bool,
}
