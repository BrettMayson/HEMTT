use crate::{symbol::Symbol, token::Token};

#[derive(Debug)]
pub enum Output {
    Direct(Token),
    Macro(Token, Vec<Self>),
}

impl Output {
    pub fn to_source(&self) -> String {
        match self {
            Self::Direct(t) => t.to_source(),
            Self::Macro(_, t) => t.iter().map(Self::to_source).collect::<String>(),
        }
    }

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
