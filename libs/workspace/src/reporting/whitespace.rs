//! Whitespace and comments

use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Whitespace characters
pub enum Whitespace {
    /// A space
    Space,
    /// A tab (\t)
    Tab,
}

impl Display for Whitespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Space => " ",
                Self::Tab => "\t",
            }
        )
    }
}
