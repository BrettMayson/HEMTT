use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A string value
pub struct Str {
    pub(crate) value: String,
    pub(crate) span: Range<usize>,
}

impl Str {
    #[must_use]
    /// Get the value
    pub fn value(&self) -> &str {
        &self.value
    }

    #[must_use]
    /// Get the span
    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}
