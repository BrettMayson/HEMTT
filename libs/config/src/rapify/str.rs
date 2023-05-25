use crate::Str;

use super::{Rapify, WriteExt};

impl Rapify for Str {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        output.write_cstring(&self.0)?;
        Ok(self.0.len() + 1)
    }

    fn rapified_length(&self) -> usize {
        self.0.len() + 1
    }

    fn rapified_code(&self) -> u8 {
        0
    }
}
