use std::{fmt::Display, sync::Arc};

use super::{Symbol, Token};

#[derive(Debug)]
/// The output of a token
pub enum Output {
    /// The token did not expand
    Direct(Arc<Token>),
    /// The token expanded to a list of tokens
    Macro(Arc<Token>, Vec<Self>),
}

impl Output {
    /// Get the last symbol of the output
    pub fn last_symbol(&self) -> Option<&Symbol> {
        match self {
            Self::Direct(t) => Some(t.symbol()),
            Self::Macro(_, t) => t.last().and_then(Self::last_symbol),
        }
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Direct(t) => t.to_string(),
                Self::Macro(_, t) => t
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<String>(),
            }
        )
    }
}

impl From<Output> for Vec<Arc<Token>> {
    fn from(value: Output) -> Self {
        match value {
            Output::Direct(t) => vec![t],
            Output::Macro(_, t) => t
                .into_iter()
                .flat_map(<Output as Into<Self>>::into)
                .collect(),
        }
    }
}
