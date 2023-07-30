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
    /// Try to evaluate a number from a string
    pub fn try_evaulation(str: &str, span: Range<usize>) -> Option<Self> {
        let value = hemtt_math::eval(str)?;
        // convert to int if possible
        if value.fract() == 0.0 {
            if value >= f64::from(i32::MIN) && value <= f64::from(i32::MAX) {
                return Some(Self::Int32 {
                    value: value as i32,
                    span,
                });
            }
            return Some(Self::Int64 {
                value: value as i64,
                span,
            });
        }
        Some(Self::Float32 {
            value: value as f32,
            span,
        })
    }

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

    #[must_use]
    /// Get the range of the number
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Int32 { span, .. } | Self::Int64 { span, .. } | Self::Float32 { span, .. } => {
                span.clone()
            }
        }
    }
}

impl ToString for Number {
    fn to_string(&self) -> String {
        match self {
            Self::Int32 { value, .. } => value.to_string(),
            Self::Int64 { value, .. } => value.to_string(),
            Self::Float32 { value, .. } => value.to_string(),
        }
    }
}
