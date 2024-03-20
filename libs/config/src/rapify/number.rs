use byteorder::{LittleEndian, WriteBytesExt};

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
            Self::Int32 { value, .. } => {
                output.write_i32::<LittleEndian>(*value)?;
                written += 4;
            }
            Self::Int64 { value, .. } => {
                output.write_i64::<LittleEndian>(*value)?;
                written += 8;
            }
            Self::Float32 { value, .. } => {
                output.write_f32::<LittleEndian>(*value)?;
                written += 4;
            }
        }
        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::Int32 { .. } | Self::Float32 { .. } => 4,
            Self::Int64 { .. } => 8,
        }
    }

    fn rapified_code(&self) -> u8 {
        match self {
            Self::Int32 { .. } => 2,
            Self::Int64 { .. } => 6,
            Self::Float32 { .. } => 1,
        }
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
        let written = Number::Int32 {
            value: 1234,
            span: 0..4,
        }
        .rapify(&mut buffer, 0)
        .unwrap();
        assert_eq!(written, 4);
        assert_eq!(buffer, vec![0xd2, 0x04, 0x00, 0x00]);
    }

    #[test]
    fn int64() {
        let mut buffer = Vec::new();
        let written = Number::Int64 {
            value: 1234,
            span: 0..4,
        }
        .rapify(&mut buffer, 0)
        .unwrap();
        assert_eq!(written, 8);
        assert_eq!(buffer, vec![0xd2, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn float32() {
        let mut buffer = Vec::new();
        let written = Number::Float32 {
            value: 1234.0,
            span: 0..4,
        }
        .rapify(&mut buffer, 0)
        .unwrap();
        assert_eq!(written, 4);
        assert_eq!(buffer, vec![0x00, 0x40, 0x9A, 0x44]);
    }
}
