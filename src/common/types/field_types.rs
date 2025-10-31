#![allow(dead_code)]

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
