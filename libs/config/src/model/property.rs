use std::ops::Range;

use crate::{Class, Ident, Value};

#[derive(Debug, Clone, PartialEq)]
/// A property of a class
pub enum Property {
    /// A property entry
    Entry {
        /// The name of the property
        name: Ident,
        /// The value of the property
        value: Value,
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
    pub const fn name(&self) -> &Ident {
        match self {
            Self::Delete(name) | Self::Entry { name, .. } => name,
            Self::Class(c) => c.name(),
            Self::MissingSemicolon(name, _) => name,
        }
    }
}
