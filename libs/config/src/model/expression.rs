use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A string value
pub struct Expression {
    pub(crate) value: String,
    pub(crate) span: Range<usize>,
}

#[cfg(feature = "serde")]
impl serde::Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}
