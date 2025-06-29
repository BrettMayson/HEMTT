use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub const fn new(value: String, span: Range<usize>) -> Self {
        Self { value, span }
    }

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

    #[must_use]
    /// Get the span of the identifier
    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}
