#[derive(Debug, Clone, PartialEq)]
/// A number value
pub enum Number {
    /// A 32-bit integer
    Int32(i32),
    /// A 64-bit integer
    Int64(i64),
    /// A 32-bit floating point number
    Float32(f32),
}

impl Number {
    #[must_use]
    /// Try to evaluate a number from a string
    pub fn try_evaluation(str: &str) -> Option<Self> {
        let value = hemtt_common::math::eval(str)?;
        // convert to int if possible
        if value.fract() == 0.0 {
            if value >= f64::from(i32::MIN) && value <= f64::from(i32::MAX) {
                return Some(Self::Int32(value as i32));
            }
            return Some(Self::Int64(value as i64));
        }
        Some(Self::Float32(value as f32))
    }

    #[must_use]
    /// Negate the number and adjust the span to include the `-`
    pub fn negate(&self) -> Self {
        match self {
            Self::Int32(value) => Self::Int32(-value),
            Self::Int64(value) => Self::Int64(-value),
            Self::Float32(value) => Self::Float32(-value),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Int32(value) => serializer.serialize_i32(*value),
            Self::Int64(value) => serializer.serialize_i64(*value),
            Self::Float32(value) => serializer.serialize_f32(*value),
        }
    }
}
