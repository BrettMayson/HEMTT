use std::iter::Sum;

use crate::{rapify::WriteExt, Array};

use super::{compressed_int_len, Rapify};

impl Rapify for Array {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = output.write_compressed_int(self.elements.len() as u32)?;

        // if self.expand {
        //     output.write_all(&[1, 0, 0, 0])?;
        //     written += 4;
        // }

        for element in &self.elements {
            output.write_all(&[element.rapified_code()])?;
            written += element.rapify(output, offset).unwrap() + 1;
        }

        assert_eq!(written, self.rapified_length());

        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        let len = compressed_int_len(self.elements.len() as u32)
            + usize::sum(self.elements.iter().map(|e| e.rapified_length() + 1));
        // if self.expand {
        //     len + 4
        // } else {
        //     len
        // }
        len
    }
}
