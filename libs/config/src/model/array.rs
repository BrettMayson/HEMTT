use std::ops::Range;

use crate::{Number, Str};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An array of entries
pub struct Array {
    pub expand: bool,
    pub items: Vec<Item>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An array value
pub enum Item {
    /// A string value
    Str(Str),
    /// A number value
    Number(Number),
    /// An array value
    Array(Vec<Item>),
    /// An invalid value
    Invalid(Range<usize>),
}
