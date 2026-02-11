use std::iter::Sum;

use byteorder::ReadBytesExt;
use chumsky::span::{SimpleSpan, Spanned};
use hemtt_common::io::{ReadExt, WriteExt, compressed_int_len};

use crate::{Array, Item, Number, Str};

use super::{Derapify, Rapify};

impl Rapify for Array {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = output.write_compressed_int(self.items.len() as u32)?;
        for item in self.items.iter() {
            output.write_all(&[item.rapified_code()])?;
            written += item.rapify(output, offset)? + 1;
        }
        assert_eq!(written, self.rapified_length());
        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        compressed_int_len(self.items.len() as u32)
            + usize::sum(self.items.iter().map(|e| e.rapified_length() + 1))
    }
}

impl Array {
    /// Derapify an array from the input stream
    ///
    /// # Errors
    /// [`std::io::Error`] if the input stream is invalid or cannot be read
    pub fn derapify<I: std::io::Read + std::io::Seek>(
        input: &mut I,
        expand: bool,
    ) -> Result<Self, std::io::Error> {
        let length = input.read_compressed_int()?;
        let mut items = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let item = Item::derapify(input)?;
            items.push(Spanned {
                inner: item,
                span: SimpleSpan::default(),
            });
        }
        Ok(Self {
            items: Spanned {
                inner: items,
                span: SimpleSpan::default(),
            },
            expand,
        })
    }
}

impl Rapify for Item {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        match self {
            Self::Str(s) => s.rapify(output, offset),
            Self::Number(n) => n.rapify(output, offset),
            Self::Array(a) => {
                let mut written = output.write_compressed_int(a.len() as u32)?;
                for item in a {
                    output.write_all(&[item.rapified_code()])?;
                    written += item.rapify(output, offset)? + 1;
                }
                Ok(written)
            }
            Self::Invalid(_) => unreachable!(),
        }
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::Str(s) => s.rapified_length(),
            Self::Number(n) => n.rapified_length(),
            Self::Array(a) => {
                compressed_int_len(a.len() as u32)
                    + usize::sum(a.iter().map(|e| e.rapified_length() + 1))
            }
            Self::Invalid(_) => unreachable!(),
        }
    }

    fn rapified_code(&self) -> u8 {
        match self {
            Self::Str(s) => s.rapified_code(),
            Self::Number(n) => n.rapified_code(),
            Self::Array(_) => 3,
            Self::Invalid(_) => unreachable!(),
        }
    }
}

impl Derapify for Item {
    fn derapify<I: std::io::Read + std::io::Seek>(input: &mut I) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let code = input.read_u8()?;
        match code {
            0 => Ok(Self::Str(Str::derapify(input)?)),
            1 => Ok(Self::Number(Number::derapify_float32(input)?)),
            2 => Ok(Self::Number(Number::derapify_int32(input)?)),
            3 => {
                let length = input.read_compressed_int()?;
                let mut items = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    let item = Self::derapify(input)?;
                    items.push(Spanned {
                        inner: item,
                        span: SimpleSpan::default(),
                    });
                }
                Ok(Self::Array(items))
            }
            6 => Ok(Self::Number(Number::derapify_int64(input)?)),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid rapified item code: {code}"),
            )),
        }
    }
}
