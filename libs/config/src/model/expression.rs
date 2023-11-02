use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A string value
pub struct Expression {
    pub(crate) value: String,
    pub(crate) span: Range<usize>,
}
