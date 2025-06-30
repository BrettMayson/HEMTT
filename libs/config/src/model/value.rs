use std::ops::Range;

use super::{Array, Expression, Number, Str};

#[derive(Debug, Clone, PartialEq)]
/// A value in a config file
pub enum Value {
    /// A string value
    ///
    /// ```cpp
    /// my_string = "Hello World";
    /// ```
    Str(Str),
    /// A number value
    ///
    /// ```cpp
    /// my_number = 1;
    /// ```
    Number(Number),
    /// An expression
    /// This is ran by the game at startup
    Expression(Expression),
    /// An array value
    ///
    /// ```cpp
    /// my_array[] = {1,2,3};
    /// ```
    Array(Array),
    /// An unexpected array value
    /// This is used when an array is found where it is not expected
    ///
    /// ```cpp
    /// my_string = {1,2,3};
    /// ```
    UnexpectedArray(Array),
    /// An invalid value
    Invalid(Range<usize>),
}

impl Value {
    #[must_use]
    /// Get the range of the value
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Str(s) => s.span.clone(),
            Self::Number(n) => n.span(),
            Self::Expression(e) => e.span.clone(),
            Self::Array(a) | Self::UnexpectedArray(a) => a.span.clone(),
            Self::Invalid(span) => span.clone(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        match self {
            Self::Str(string) => string.serialize(serializer),
            Self::Number(number) => number.serialize(serializer),
            Self::Expression(expression) => expression.serialize(serializer),
            Self::Array(array) | Self::UnexpectedArray(array) => array.serialize(serializer),
            Self::Invalid(..) => serializer.serialize_none(),
        }
    }
}
