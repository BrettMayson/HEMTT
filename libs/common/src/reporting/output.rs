use super::{Symbol, Token};

#[derive(Debug)]
/// The output of a token
pub enum Output {
    /// The token did not expand
    Direct(Token),
    /// The token expanded to a list of tokens
    Macro(Token, Vec<Self>),
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

impl ToString for Output {
    fn to_string(&self) -> String {
        match self {
            Self::Direct(t) => t.to_string(),
            Self::Macro(_, t) => t
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<String>(),
        }
    }
}

impl From<Output> for Vec<Token> {
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
