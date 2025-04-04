use crate::Value;

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(str) => write!(f, "{str}"),
            Self::Number(number) => write!(f, "{number}"),
            Self::Expression(expression) => write!(f, "{expression}"),
            Self::Array(array) | Self::UnexpectedArray(array) => write!(f, "{array}"),
            Self::Invalid(_) => unreachable!(),
        }
    }
}
