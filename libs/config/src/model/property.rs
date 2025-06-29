use std::ops::Range;

use crate::{Class, Ident, Value};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
/// A property of a class
pub enum Property {
    /// A property entry
    Entry {
        /// The name of the property
        name: Ident,
        /// The value of the property
        value: Value,
        /// An array was expected
        expected_array: bool,
    },
    /// A sub-class
    Class(Class),
    /// A class deletion
    Delete(Ident),
    /// A property that is missing a semicolon
    MissingSemicolon(Ident, Range<usize>),
}

impl Property {
    #[must_use]
    /// Get the name of the property
    ///
    /// # Panics
    /// If this is a [`Class::Root`], which should never occur
    pub const fn name(&self) -> &Ident {
        match self {
            Self::Class(c) => c.name().expect("root should not be a property"),
            Self::MissingSemicolon(name, _) | Self::Delete(name) | Self::Entry { name, .. } => name,
        }
    }

    #[must_use]
    /// Is the property a class
    pub const fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }
}
