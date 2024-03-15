//! Whitespace and comments

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Whitespace characters
pub enum Whitespace {
    /// A space
    Space,
    /// A tab (\t)
    Tab,
}

impl ToString for Whitespace {
    fn to_string(&self) -> String {
        match self {
            Self::Space => " ",
            Self::Tab => "\t",
        }
        .to_string()
    }
}
