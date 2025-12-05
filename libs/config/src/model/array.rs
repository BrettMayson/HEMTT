use std::ops::Range;

use crate::{Number, Str};

#[derive(Debug, Clone, PartialEq)]
/// An array of entries
pub struct Array {
    pub(crate) expand: bool,
    pub(crate) items: Vec<Item>,
    pub(crate) span: Range<usize>,
}

impl Array {
    #[cfg(test)]
    #[must_use]
    pub fn test_new(items: Vec<Item>) -> Self {
        let span = items.first().map_or(0..0, Item::span);
        Self {
            expand: false,
            items,
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// An array value
pub enum Item {
    /// A string value
    Str(Str),
    /// A number value
    Number(Number),
    /// An array value
    Array(Vec<Self>),
    /// An invalid value
    Invalid(Range<usize>),
}

impl Item {
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Str(s) => s.span.clone(),
            Self::Number(n) => n.span(),
            Self::Array(items) => {
                if let (Some(first), Some(last)) = (items.first(), items.last()) {
                    first.span().start..last.span().end
                } else {
                    0..0
                }
            }
            Self::Invalid(span) => span.clone(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Array {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Str(str) => str.serialize(serializer),
            Self::Number(number) => number.serialize(serializer),
            Self::Array(items) => {
                use serde::ser::SerializeSeq;
                let mut state = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    state.serialize_element(item)?;
                }
                state.end()
            }
            Self::Invalid(_) => serializer.serialize_none(),
        }
    }
}
