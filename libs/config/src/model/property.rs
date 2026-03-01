use chumsky::span::{SimpleSpan, Spanned};

use crate::{Class, Ident, Value};

#[derive(Debug, Clone, PartialEq)]
/// A property of a class
pub enum Property {
    /// A property entry
    Entry {
        /// The name of the property
        name: Spanned<Ident>,
        /// The value of the property
        value: Spanned<Value>,
        /// An array was expected
        expected_array: bool,
    },
    /// A sub-class
    Class(Spanned<Class>),
    /// A class deletion
    Delete(Spanned<Ident>),
    /// A property that is missing a semicolon
    MissingSemicolon(Spanned<Ident>),
    /// Extra semicolons
    ExtraSemicolons(SimpleSpan),
}

impl Property {
    #[must_use]
    /// Get the name of the property
    ///
    /// # Panics
    /// If this is a [`Class::Root`], which should never occur
    pub fn name(&self) -> Option<&Spanned<Ident>> {
        match self {
            Self::Class(c) => Some(c.name().expect("root should not be a property")),
            Self::MissingSemicolon(name) | Self::Delete(name) | Self::Entry { name, .. } => {
                Some(name)
            }
            Self::ExtraSemicolons(_) => None,
        }
    }

    #[must_use]
    /// Is the property a class
    pub const fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Property {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Entry { value, .. } => value.serialize(serializer),
            Self::Class(class) => class.serialize(serializer),
            Self::MissingSemicolon(..) | Self::Delete(..) | Self::ExtraSemicolons(..) => {
                serializer.serialize_unit()
            }
        }
    }
}
