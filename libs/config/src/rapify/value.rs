use crate::{Expression, Number, Str, Value};

use super::{Derapify, Rapify};

impl Rapify for Value {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let written = match self {
            Self::Str(s) => s.rapify(output, offset),
            Self::Number(n) => n.rapify(output, offset),
            Self::Expression(e) => e.rapify(output, offset),
            Self::Array(a) => a.rapify(output, offset),
            Self::UnexpectedArray(_) | Self::Invalid(_) => unreachable!(),
        }?;
        assert_eq!(written, self.rapified_length());
        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::Str(s) => s.rapified_length(),
            Self::Number(n) => n.rapified_length(),
            Self::Expression(e) => e.rapified_length(),
            Self::Array(a) => a.rapified_length(),
            Self::UnexpectedArray(_) | Self::Invalid(_) => unreachable!(),
        }
    }

    fn rapified_code(&self) -> u8 {
        match self {
            Self::Str(s) => s.rapified_code(),
            Self::Number(n) => n.rapified_code(),
            Self::Expression(e) => e.rapified_code(),
            Self::Array(a) => a.rapified_code(),
            Self::UnexpectedArray(_) | Self::Invalid(_) => unreachable!(),
        }
    }
}

impl Value {
    /// Derapify a value from the input stream
    ///
    /// # Errors
    /// [`std::io::Error`] if the input stream is invalid or cannot be read
    pub fn derapify<I: std::io::Read + std::io::Seek>(
        input: &mut I,
        subcode: u8,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        match subcode {
            0 => Ok(Self::Str(Str::derapify(input)?)),
            1 => Ok(Self::Number(Number::derapify_float32(input)?)),
            2 => Ok(Self::Number(Number::derapify_int32(input)?)),
            4 => Ok(Self::Expression(Expression::derapify(input)?)),
            6 => Ok(Self::Number(Number::derapify_int64(input)?)),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid rapified value code: {subcode}"),
            )),
        }
    }
}
