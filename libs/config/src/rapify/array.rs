use std::iter::Sum;

use hemtt_io::{compressed_int_len, WriteExt};

use crate::{Array, Item};

use super::Rapify;

impl Rapify for Array {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = output.write_compressed_int(self.items.len() as u32)?;
        for item in &self.items {
            output.write_all(&[item.rapified_code()])?;
            written += item.rapify(output, offset).unwrap() + 1;
        }
        assert_eq!(written, self.rapified_length());
        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        compressed_int_len(self.items.len() as u32)
            + usize::sum(self.items.iter().map(|e| e.rapified_length() + 1))
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
                    written += item.rapify(output, offset).unwrap() + 1;
                }
                Ok(written)
            }
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
        }
    }

    fn rapified_code(&self) -> u8 {
        match self {
            Self::Str(s) => s.rapified_code(),
            Self::Number(n) => n.rapified_code(),
            Self::Array(_) => 3,
        }
    }
}
