use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A string value
pub struct Expression(pub(crate) Arc<str>);

impl Expression {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.value())
    }
}
