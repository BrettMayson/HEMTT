use crate::Number;

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int32(value) => {
                write!(f, "{value}")
            }
            Self::Int64(value) => {
                write!(f, "{value}")
            }
            Self::Float32(value) => {
                write!(f, "{value}")
            }
        }
    }
}
