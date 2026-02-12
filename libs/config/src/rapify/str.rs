use std::sync::Arc;

use hemtt_common::io::{ReadExt, WriteExt};

use crate::Str;

use super::{Derapify, Rapify};

impl Rapify for Str {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        output.write_cstring(self.value())?;
        Ok(self.value().len() + 1)
    }

    fn rapified_length(&self) -> usize {
        self.value().len() + 1
    }

    fn rapified_code(&self) -> u8 {
        0
    }
}

impl Derapify for Str {
    fn derapify<I: std::io::Read + std::io::Seek>(input: &mut I) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self(Arc::from(input.read_cstring()?)))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use crate::Str;

    use super::Rapify;

    #[test]
    fn str() {
        let mut buffer = Vec::new();
        let written = Str(Arc::from("Hello World"))
            .rapify(&mut buffer, 0)
            .unwrap();
        assert_eq!(written, 12);
        assert_eq!(
            buffer,
            vec![
                0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x00
            ]
        );
    }
}
