use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::Number;

use super::Rapify;

impl Rapify for Number {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = 0;
        match self {
            Self::Int32(value) => {
                output.write_i32::<LittleEndian>(*value)?;
                written += 4;
            }
            Self::Int64(value) => {
                output.write_i64::<LittleEndian>(*value)?;
                written += 8;
            }
            Self::Float32(value) => {
                output.write_f32::<LittleEndian>(*value)?;
                written += 4;
            }
        }
        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::Int32(_) | Self::Float32(_) => 4,
            Self::Int64(_) => 8,
        }
    }

    fn rapified_code(&self) -> u8 {
        match self {
            Self::Int32(_) => 2,
            Self::Int64(_) => 6,
            Self::Float32(_) => 1,
        }
    }
}

impl Number {
    /// Derapify an int32 value from the input stream
    ///
    /// # Errors
    /// [`std::io::Error`] if the input stream is invalid or cannot be read
    pub fn derapify_int32<I: std::io::Read + std::io::Seek>(
        input: &mut I,
    ) -> Result<Self, std::io::Error> {
        Ok(Self::Int32(input.read_i32::<LittleEndian>()?))
    }

    /// Derapify an int64 value from the input stream
    ///
    /// # Errors
    /// [`std::io::Error`] if the input stream is invalid or cannot be read
    pub fn derapify_int64<I: std::io::Read + std::io::Seek>(
        input: &mut I,
    ) -> Result<Self, std::io::Error> {
        Ok(Self::Int64(input.read_i64::<LittleEndian>()?))
    }

    /// Derapify a float32 value from the input stream
    ///
    /// # Errors
    /// [`std::io::Error`] if the input stream is invalid or cannot be read
    pub fn derapify_float32<I: std::io::Read + std::io::Seek>(
        input: &mut I,
    ) -> Result<Self, std::io::Error> {
        Ok(Self::Float32(input.read_f32::<LittleEndian>()?))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::Number;

    use super::Rapify;

    #[test]
    fn int32() {
        let mut buffer = Vec::new();
        let written = Number::Int32(1234).rapify(&mut buffer, 0).unwrap();
        assert_eq!(written, 4);
        assert_eq!(buffer, vec![0xd2, 0x04, 0x00, 0x00]);
    }

    #[test]
    fn int64() {
        let mut buffer = Vec::new();
        let written = Number::Int64(1234).rapify(&mut buffer, 0).unwrap();
        assert_eq!(written, 8);
        assert_eq!(buffer, vec![0xd2, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn float32() {
        let mut buffer = Vec::new();
        let written = Number::Float32(1234.0).rapify(&mut buffer, 0).unwrap();
        assert_eq!(written, 4);
        assert_eq!(buffer, vec![0x00, 0x40, 0x9A, 0x44]);
    }
}
