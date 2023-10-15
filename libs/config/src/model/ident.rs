use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// An identifier
///
/// ```cpp
/// my_ident = 1;
/// ```
///
/// ```cpp
/// class my_ident {
///    ...
/// };
/// ```
pub struct Ident {
    /// Identifier value
    pub value: String,
    /// Identifier span
    pub span: Range<usize>,
}

impl Ident {
    #[must_use]
    /// Get the value of the identifier
    pub fn as_str(&self) -> &str {
        &self.value
    }

    #[must_use]
    /// Get the length of the identifier
    pub fn len(&self) -> usize {
        self.value.len()
    }

    #[must_use]
    /// Check if the identifier is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}
