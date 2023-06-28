use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
/// A number value
pub enum Number {
    /// A 32-bit integer
    Int32 {
        /// Number value
        value: i32,
        /// Number span
        span: Range<usize>,
    },
    /// A 64-bit integer
    Int64 {
        /// Number value
        value: i64,
        /// Number span
        span: Range<usize>,
    },
    /// A 32-bit floating point number
    Float32 {
        /// Number value
        value: f32,
        /// Number span
        span: Range<usize>,
    },
}

impl Number {
    #[must_use]
    /// Negate the number
    pub fn negate(&self) -> Self {
        match self {
            Self::Int32 { value, span } => Self::Int32 {
                value: -value,
                span: span.clone(),
            },
            Self::Int64 { value, span } => Self::Int64 {
                value: -value,
                span: span.clone(),
            },
            Self::Float32 { value, span } => Self::Float32 {
                value: -*value,
                span: span.clone(),
            },
        }
    }
}
