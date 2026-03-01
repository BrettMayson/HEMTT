use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A string value
pub struct Str(pub(crate) Arc<str>);

impl Str {
    #[must_use]
    /// Get the value
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.value())
    }
}
