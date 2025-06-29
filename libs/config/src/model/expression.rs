use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A string value
pub struct Expression {
    pub(crate) value: String,
    pub(crate) span: Range<usize>,
}
