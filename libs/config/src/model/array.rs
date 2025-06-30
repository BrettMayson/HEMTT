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

#[cfg(feature = "serde")]
impl serde::Serialize for Array {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        use serde::ser::SerializeSeq;
        let mut state = serializer.serialize_seq(Some(self.items.len()))?;
        for item in &self.items {
            state.serialize_element(&item)?;
        }
        state.end()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        match self {
            Item::Str(str) => str.serialize(serializer),
            Item::Number(number) => number.serialize(serializer),
            Item::Array(items) => {
                use serde::ser::SerializeSeq;
                let mut state = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    state.serialize_element(item)?;
                }
                state.end()
            },
            Item::Invalid(_) => serializer.serialize_none()
        }
    }
}