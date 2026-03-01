use std::sync::Arc;

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
pub struct Ident(pub(crate) Arc<str>);

impl Ident {
    #[must_use]
    pub fn new(value: &str) -> Self {
        Self(value.into())
    }

    #[must_use]
    /// Get the value of the identifier
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    /// Get the length of the identifier
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    /// Check if the identifier is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
