use hemtt_common::io::{ReadExt, WriteExt};

use crate::Expression;

use super::{Derapify, Rapify};

impl Rapify for Expression {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        output.write_cstring(&self.value)?;
        Ok(self.value.len() + 1)
    }

    fn rapified_length(&self) -> usize {
        self.value.len() + 1
    }

    fn rapified_code(&self) -> u8 {
        4
    }
}

impl Derapify for Expression {
    fn derapify<I: std::io::Read + std::io::Seek>(input: &mut I) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let start = input.stream_position()? as usize;
        let value = input.read_cstring()?;
        Ok(Self {
            value,
            span: start..input.stream_position()? as usize,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::Expression;

    use super::Rapify;

    #[test]
    fn str() {
        let mut buffer = Vec::new();
        let written = Expression {
            value: "getResolution".to_string(),
            span: 0..14,
        }
        .rapify(&mut buffer, 0)
        .unwrap();
        assert_eq!(written, 14);
        assert_eq!(
            buffer,
            vec![
                103, 101, 116, 82, 101, 115, 111, 108, 117, 116, 105, 111, 110, 0
            ]
        );
    }
}
