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
            Self::Int32(i) => {
                output.write_i32::<LittleEndian>(*i)?;
                written += 4;
            }
            Self::Int64(i) => {
                output.write_i64::<LittleEndian>(*i)?;
                written += 8;
            }
            Self::Float32(f) => {
                output.write_f32::<LittleEndian>(*f)?;
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
