use std::ops::Range;

use crate::{Number, Str};

#[derive(Debug, Clone, PartialEq)]
/// An array of entries
pub struct Array {
    pub(crate) expand: bool,
    pub(crate) items: Vec<Item>,
    pub(crate) span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
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
